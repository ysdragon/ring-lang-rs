# parsec.ring - Generate Rust Ring bindings from configuration files
# 
# Usage: ring parsec.ring input.rf output.rs [output.ring]
#
# Author: Youssef Saeed (ysdragon)
# Based on: parsec.ring by Mahmoud Fayed
# Purpose: Generate Ring bindings for Rust crates
#
# Configuration File Format (.rf):
#   <meta>           - Library metadata (crate_name, lib_prefix)
#   <code>           - Raw Rust code to include (auto-detects pub fn)
#   <functions>      - Function prototypes (explicit declaration)
#   <struct>         - Struct definitions with auto-generated accessors
#   <impl>           - Impl block methods
#   <constants>      - Constants to expose
#   <register>       - Manual function registration for ring_func! macros
#   <filter>         - Conditional inclusion
#   <comment>        - Comments (ignored)
#   <loadfile>       - Include another .rf file
#   <runcodenow>     - Execute Ring code during parsing

load "stdlibcore.ring"

# Load parser library from the same directory as this script
cScriptPath = sysargv[2]
cScriptDir = ""
nLastSlash = 0
for i = 1 to len(cScriptPath)
    c = cScriptPath[i]
    if c = "/" or c = char(92)
        nLastSlash = i
    ok
next
if nLastSlash > 0
    cScriptDir = left(cScriptPath, nLastSlash)
ok
eval('load "' + cScriptDir + 'parsec_lib.ring"')

# ==============================================================================
# Constants
# ==============================================================================

# Instruction types
C_INS_FUNCTION   = 1
C_INS_CODE       = 2
C_INS_STRUCT     = 3
C_INS_IMPL       = 4
C_INS_CONSTANT   = 5
C_INS_FILTER     = 6
C_INS_COMMENT    = 7
C_INS_RUNCODE    = 8
C_INS_META       = 9
C_INS_REGISTER   = 10

# Rust type categories
C_TYPE_VOID      = 1
C_TYPE_NUMBER    = 2
C_TYPE_STRING    = 3
C_TYPE_POINTER   = 4
C_TYPE_LIST      = 5
C_TYPE_RESULT    = 6
C_TYPE_OPTION    = 7
C_TYPE_BOOL      = 8
C_TYPE_STRUCT    = 9
C_TYPE_UNKNOWN   = 10

# Tabs
C_TABS_1 = "    "
C_TABS_2 = "        "
C_TABS_3 = "            "
C_TABS_4 = "                "

# ==============================================================================
# Global Variables
# ==============================================================================

# Metadata
$cCrateName     = ""
$cLibPrefix     = ""
$cCurrentStruct = ""
$cCurrentImpl   = ""

# Type lists
$aNumberTypes = ["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", 
                 "f32", "f64", "isize", "usize", "c_int", "c_uint", 
                 "c_long", "c_ulong", "c_float", "c_double"]

$aStringTypes = ["&str", "String", "&String", "&'static str"]

$aListTypes   = ["Vec", "&[", "&mut ["]

# Collected data
$aStructs    = []   # List of struct definitions
$aFunctions  = []   # List of standalone functions
$aImplBlocks = []   # List of impl blocks with methods

# For Ring class generation
$aClassesList = []

# Custom new() constructors from impl blocks
# Format: [ [struct_name, [[param_name, param_type], ...], return_type], ... ]
$aCustomConstructors = []

# Global list for runcodenow
$globals = []

# Ring reserved keywords - cannot be used as method names
# 56 keywords + 14 alias keywords
$aRingKeywords = [
    # Core keywords (56)
    "again", "and", "but", "bye", "call", "case", "catch", "class", "def", "do",
    "done", "else", "elseif", "end", "exit", "for", "foreach", "from", "func", "function",
    "get", "give", "if", "import", "in", "load", "loop", "new", "next", "not",
    "off", "ok", "on", "or", "other", "package", "private", "put", "return", "see",
    "step", "switch", "to", "try", "while", "endfunc", "endclass", "endpackage", "endif", "endfor",
    "endwhile", "endswitch", "endtry", "endfunction", "break", "continue",
    # Alias keywords (14)
    "this", "self", "super", "main", "init", "operator", "bracestart", "braceexpreval",
    "bracenewline", "braceerror", "braceend", "ringvm_see", "ringvm_give", "ringvm_errorhandler",
    # Ring-specific
    "changeringkeyword", "changeringoperator", "loadsyntax"
]

# ==============================================================================
# Main Entry Point
# ==============================================================================

Func Main
    if len(sysargv) < 4
        ? "Ring Rust Codegen - Generate Ring bindings for Rust"
        ? ""
        ? "Usage: ring parsec.ring input.rf output.rs [output.ring]"
        ? ""
        ? "Arguments:"
        ? "  input.rf    - Configuration file with Rust function/struct definitions"
        ? "  output.rs   - Generated Rust source file"
        ? "  output.ring - (Optional) Generated Ring class wrappers"
        bye
    ok

    cInputFile  = sysargv[3]
    cOutputRust = sysargv[4]
    cOutputRing = ""
    
    if len(sysargv) >= 5
        cOutputRing = sysargv[5]
    ok

    ? "Ring Rust Codegen"
    ? "================="
    ? "Input:  " + cInputFile
    ? "Output: " + cOutputRust
    if cOutputRing != ""
        ? "Ring:   " + cOutputRing
    ok
    ? ""

    # Read and parse the configuration file
    if not fexists(cInputFile)
        ? "Error: Input file not found: " + cInputFile
        bye
    ok

    cFileContent = read(cInputFile)
    aLines = str2list(cFileContent)

    # Parse the configuration
    aData = []
    cDir = currentdir()
    chdir(JustFilePath(cInputFile))
    ParseConfigFile(aData, aLines)
    chdir(cDir)

    # Generate Rust code
    cRustCode = GenerateRustCode(aData)
    WriteFile(cOutputRust, cRustCode)

    # Generate Ring classes (optional)
    if cOutputRing != ""
        cRingCode = GenerateRingClasses(aData)
        WriteFile(cOutputRing, cRingCode)
    ok

    ? ""
    ? "Done!"

# ==============================================================================
# Parsing Functions
# ==============================================================================

Func ParseConfigFile aData, aLines
    nFlag = C_INS_FUNCTION
    nFilterFlag = C_INS_FUNCTION
    nMax = len(aLines)
    cStructData = ""
    cImplData = ""
    
    for t = 1 to nMax
        cLine = aLines[t]
        
        # Don't trim lines in code blocks - preserve indentation
        if nFlag != C_INS_CODE
            cLine = trim(cLine)
        ok
        
        # Skip empty lines outside of code blocks
        if cLine = "" and nFlag != C_INS_CODE
            loop
        ok

        # Handle comments (# at start of line)
        if left(trim(cLine), 1) = "#" and nFlag != C_INS_CODE
            loop
        ok

        # Skip comment blocks
        if nFlag = C_INS_COMMENT and lower(cLine) != "</comment>"
            loop
        ok

        # Skip filtered sections
        if nFlag = C_INS_FILTER and lower(cLine) != "</filter>"
            loop
        ok

        ? "Processing: " + cLine

        # Section markers
        switch lower(cLine)
        on "<meta>"
            nFlag = C_INS_META
            loop
        on "</meta>"
            nFlag = C_INS_FUNCTION
            loop
        on "<code>"
            nFlag = C_INS_CODE
            loop
        on "</code>"
            nFlag = C_INS_FUNCTION
            loop
        on "<functions>"
            nFlag = C_INS_FUNCTION
            loop
        on "</functions>"
            nFlag = C_INS_FUNCTION
            loop
        on "<struct>"
            nFlag = C_INS_STRUCT
            cStructData = ""
            loop
        on "</struct>"
            if cStructData != ""
                aData + [C_INS_STRUCT, cStructData]
            ok
            nFlag = C_INS_FUNCTION
            cStructData = ""
            loop
        on "<impl>"
            nFlag = C_INS_IMPL
            cImplData = ""
            loop
        on "</impl>"
            if cImplData != ""
                aData + [C_INS_IMPL, cImplData]
            ok
            nFlag = C_INS_FUNCTION
            cImplData = ""
            loop
        on "<constants>"
            nFlag = C_INS_CONSTANT
            loop
        on "</constants>"
            nFlag = C_INS_FUNCTION
            loop
        on "<register>"
            nFlag = C_INS_REGISTER
            loop
        on "</register>"
            nFlag = C_INS_FUNCTION
            loop
        on "<comment>"
            nFlag = C_INS_COMMENT
            loop
        on "</comment>"
            nFlag = C_INS_FUNCTION
            loop
        on "</filter>"
            nFlag = nFilterFlag
            loop
        off

        # Handle <filter> with condition
        if left(lower(cLine), 8) = "<filter>"
            cFilter = "lInclude = (" + trim(substr(cLine, 9)) + ")"
            ? "Execute Filter: " + cFilter
            eval(cFilter)
            ? "Filter result: " + lInclude
            nFilterFlag = nFlag
            if lInclude = false
                nFlag = C_INS_FILTER
            ok
            loop
        ok

        # Handle <loadfile>
        if left(lower(cLine), 10) = "<loadfile>"
            cSubFileName = trim(substr(cLine, 11))
            if fexists(cSubFileName)
                cSubFileStr = read(cSubFileName)
                aSubList = str2list(cSubFileStr)
                cDir = currentdir()
                chdir(JustFilePath(cSubFileName))
                ParseConfigFile(aData, aSubList)
                chdir(cDir)
            else
                ? "Warning: Include file not found: " + cSubFileName
            ok
            loop
        ok

        # Handle <runcodenow>
        if left(lower(cLine), 12) = "<runcodenow>"
            cCodeNow = trim(substr(cLine, 13))
            eval(cCodeNow)
            loop
        ok

        # Process line based on current section
        switch nFlag
        on C_INS_META
            ProcessMetaLine(cLine)
        on C_INS_CODE
            aData + [C_INS_CODE, cLine]
            # Auto-detect pub fn signatures for wrapping (standalone functions only)
            cTrimmed = trim(cLine)
            if left(cTrimmed, 7) = "pub fn "
                # Skip impl methods:
                # - contain &self or &mut self (instance methods)
                # - return Self (constructors)
                lIsMethod = substr(cTrimmed, "&self") > 0 or 
                            substr(cTrimmed, "&mut self") > 0 or
                            substr(cTrimmed, "-> Self") > 0
                if not lIsMethod
                    # Extract signature up to the opening brace
                    nBracePos = substr(cTrimmed, "{")
                    if nBracePos > 0
                        cSig = trim(left(cTrimmed, nBracePos - 1))
                        # Remove "pub " prefix to get "fn name(...) -> Type"
                        cSig = trim(substr(cSig, 5))
                        aData + [C_INS_FUNCTION, ParseFunctionSignature(cSig)]
                    ok
                ok
            ok
        on C_INS_FUNCTION
            if left(lower(cLine), 3) = "fn "
                aData + [C_INS_FUNCTION, ParseFunctionSignature(cLine)]
            ok
        on C_INS_STRUCT
            cStructData += cLine + nl
        on C_INS_IMPL
            cImplData += cLine + nl
        on C_INS_CONSTANT
            aData + [C_INS_CONSTANT, ParseConstant(cLine)]
        on C_INS_REGISTER
            # Parse manual registration: ring_name => rust_func_name
            # or just: name (assumes json_name => ring_json_name)
            aData + [C_INS_REGISTER, ParseRegister(cLine)]
        off
    next

Func ProcessMetaLine cLine
    aResult = ParseMetaLine(cLine)
    if aResult[1] != ""
        switch lower(aResult[1])
        on "crate_name"
            ? "  Crate name: " + $cCrateName
        on "lib_prefix"
            ? "  Library prefix: " + $cLibPrefix
        off
    ok

Func ParseRegister cLine
    return ParseRegisterLine(cLine)

Func ParseFunctionSignature cLine
    aResult = ParseFuncSignature(cLine)
    if aResult[1] = "" and trim(cLine) != ""
        ? "Error: Invalid function signature: " + cLine
    ok
    return aResult

Func ParseParameters cParams
    return ParseFuncParams(cParams)

Func ParseConstant cLine
    return ParseConstantDef(cLine)

Func ParseStruct cStructData
    # Extended version that also extracts attributes
    # Returns: [name, [[field_name, field_type], ...], [attributes]]
    
    # Get basic struct info from parser
    aBasic = ParseStructDef(cStructData)
    cStructName = aBasic[1]
    aFields = aBasic[2]
    
    # Extract attributes
    aAttributes = []
    aLines = str2list(cStructData)
    for cLine in aLines
        cLine = trim(cLine)
        if left(cLine, 1) = "#"
            aAttributes + cLine
        ok
    next
    
    return [cStructName, aFields, aAttributes]

Func ParseImpl cImplData
    # Extended version with is_static flag and custom constructor handling
    # Returns: [struct_name, [[method_name, params, return_type, is_static], ...]]
    
    # Get basic impl info from parser
    aBasic = ParseImplDef(cImplData)
    cStructName = aBasic[1]
    aBasicMethods = aBasic[2]
    
    # Process methods: detect constructors, add is_static flag
    aMethods = []
    for aMethod in aBasicMethods
        cMethodName = aMethod[1]
        aParams = aMethod[2]
        cReturnType = aMethod[3]
        
        # Check if this is a new() constructor returning Self
        if cMethodName = "new" and (cReturnType = "Self" or cReturnType = cStructName)
            $aCustomConstructors + [cStructName, aParams, cReturnType]
            loop
        ok
        
        # Check if static (no &self or &mut self in first param)
        lStatic = true
        if len(aParams) > 0
            cFirstParam = trim(aParams[1][1])
            if cFirstParam = "self" or cFirstParam = "&self" or 
               cFirstParam = "&mut self" or cFirstParam = "& self" or
               cFirstParam = "& mut self" or substr(cFirstParam, "self")
                lStatic = false
                # Remove self from params for code generation
                del(aParams, 1)
            ok
        ok
        aMethods + [cMethodName, aParams, cReturnType, lStatic]
    next
    
    return [cStructName, aMethods]

# ==============================================================================
# Type Classification
# ==============================================================================

Func GetRustTypeCategory cType
    cType = trim(cType)
    
    # Void
    if cType = "" or cType = "()"
        return C_TYPE_VOID
    ok
    
    # Result<T, E>
    if left(cType, 7) = "Result<"
        return C_TYPE_RESULT
    ok
    
    # Option<T>
    if left(cType, 7) = "Option<"
        return C_TYPE_OPTION
    ok
    
    # Vec<T> and slices
    if left(cType, 4) = "Vec<" or left(cType, 2) = "&[" or left(cType, 1) = "["
        return C_TYPE_LIST
    ok
    
    # Bool
    if cType = "bool"
        return C_TYPE_BOOL
    ok
    
    # Numbers (exact match only)
    for cNumType in $aNumberTypes
        if cType = cNumType
            return C_TYPE_NUMBER
        ok
    next
    
    # Strings (exact match only, no substring)
    if cType = "String" or cType = "&str" or cType = "&String" or cType = "str"
        return C_TYPE_STRING
    ok
    
    # Pointers
    if left(cType, 5) = "*mut " or left(cType, 7) = "*const " or 
       left(cType, 5) = "&mut " or left(cType, 1) = "&"
        return C_TYPE_POINTER
    ok
    
    # Struct types (starts with uppercase letter, not a pointer)
    if IsStructType(cType)
        return C_TYPE_STRUCT
    ok
    
    # Default to unknown
    return C_TYPE_UNKNOWN

Func GetInnerType cType
    # Extract inner type from Option<T>, Result<T, E>, Vec<T>, etc.
    nStart = substr(cType, "<")
    if nStart = 0 return cType ok
    
    nEnd = len(cType)
    if right(cType, 1) = ">"
        nEnd = len(cType) - 1
    ok
    
    cInner = substr(cType, nStart + 1, nEnd - nStart)
    
    # For Result<T, E>, get just T
    nComma = substr(cInner, ",")
    if nComma > 0
        cInner = trim(left(cInner, nComma - 1))
    ok
    
    return cInner

Func GetPointerInnerType cType
    # Extract type from *mut T, *const T, &T, &mut T
    cType = trim(cType)
    if left(cType, 5) = "*mut "
        return trim(substr(cType, 6))
    ok
    if left(cType, 7) = "*const "
        return trim(substr(cType, 8))
    ok
    if left(cType, 5) = "&mut "
        return trim(substr(cType, 6))
    ok
    if left(cType, 1) = "&"
        return trim(substr(cType, 2))
    ok
    return cType

# ==============================================================================
# Rust Code Generation
# ==============================================================================

Func GenerateRustCode aData
    cCode = ""
    
    # Header
    cCode += "// Auto-generated Ring extension" + nl
    cCode += "// Generated by parsec.ring" + nl
    cCode += "// Do not edit manually" + nl
    cCode += nl
    cCode += "#![allow(non_snake_case)]" + nl
    cCode += "#![allow(unused_variables)]" + nl
    cCode += "#![allow(unused_imports)]" + nl
    cCode += nl
    cCode += "use ring_lang_rs::*;" + nl
    cCode += "use std::ffi::c_void;" + nl
    
    if $cCrateName != ""
        cCode += "use " + $cCrateName + "::*;" + nl
    ok
    
    cCode += nl
    
    # First pass: collect all impl method names to avoid conflicts
    aImplMethods = []  # List of [struct_name, method_name]
    for aEntry in aData
        if aEntry[1] = C_INS_IMPL
            aImpl = ParseImpl(aEntry[2])
            if aImpl[1] != ""
                for aMethod in aImpl[2]
                    aImplMethods + [aImpl[1], aMethod[1]]
                next
            ok
        ok
    next
    
    # Process all data entries in two passes:
    # Pass 1: Output all code blocks first
    # Pass 2: Generate wrappers for functions, structs, impls, constants
    
    aFunctionsList = []
    aProcessedFuncs = []   # Track processed function names to avoid duplicates
    
    # Pass 1: Output code blocks
    for aEntry in aData
        if aEntry[1] = C_INS_CODE
            cCode += aEntry[2] + nl
        ok
    next
    
    # Pass 2: Generate wrappers
    for aEntry in aData
        switch aEntry[1]
        on C_INS_FUNCTION
            aFunc = aEntry[2]
            if aFunc[1] != "" and find(aProcessedFuncs, aFunc[1]) = 0
                aProcessedFuncs + aFunc[1]
                cCode += GenerateFunctionWrapper(aFunc)
                aReg = GetFunctionRegistration(aFunc)
                aFunctionsList + [aReg[1], aReg[2]]
            ok
        on C_INS_STRUCT
            aStruct = ParseStruct(aEntry[2])
            if aStruct[1] != ""
                cCode += GenerateStructWrappers(aStruct, aImplMethods)
                aStructRegs = GetStructRegistrations(aStruct, aImplMethods)
                for aReg in aStructRegs
                    aFunctionsList + [aReg[1], aReg[2]]
                next
                $aClassesList + [aStruct[1], aStruct[2]]
            ok
        on C_INS_IMPL
            aImpl = ParseImpl(aEntry[2])
            if aImpl[1] != ""
                cCode += GenerateImplWrappers(aImpl)
                aImplRegs = GetImplRegistrations(aImpl)
                for aReg in aImplRegs
                    aFunctionsList + [aReg[1], aReg[2]]
                next
            ok
        on C_INS_CONSTANT
            aConst = aEntry[2]
            if aConst[1] != ""
                cCode += GenerateConstantGetter(aConst)
                aReg = GetConstantRegistration(aConst)
                aFunctionsList + [aReg[1], aReg[2]]
            ok
        on C_INS_REGISTER
            aReg = aEntry[2]
            if aReg[1] != "" and aReg[2] != ""
                aFunctionsList + [aReg[1], aReg[2]]
            ok
        off
    next
    
    # Generate ring_libinit
    cCode += GenerateLibInit(aFunctionsList)
    
    return cCode

Func GenerateFunctionWrapper aFunc
    # aFunc = [name, [[param_name, param_type], ...], return_type]
    
    cFuncName = aFunc[1]
    aParams = aFunc[2]
    cReturnType = aFunc[3]
    
    cWrapperName = "ring_" + $cLibPrefix
    if $cLibPrefix != ""
        cWrapperName += "_"
    ok
    cWrapperName += cFuncName
    
    nParamCount = len(aParams)
    
    cCode = nl
    cCode += "// ============================================" + nl
    cCode += "// Function: " + cFuncName + nl
    cCode += "// ============================================" + nl
    cCode += "ring_func!(" + cWrapperName + ", |p| {" + nl
    
    # Parameter count check
    cCode += C_TABS_1 + "ring_check_paracount!(p, " + nParamCount + ");" + nl
    
    # Parameter type checks
    for i = 1 to nParamCount
        cCode += GenerateParamCheck(aParams[i][2], i)
    next
    
    cCode += nl
    
    # Get parameters
    for i = 1 to nParamCount
        cCode += GenerateParamGet(aParams[i][1], aParams[i][2], i)
    next
    
    # Call function
    cCode += nl
    nReturnCategory = GetRustTypeCategory(cReturnType)
    
    if nReturnCategory = C_TYPE_VOID
        cCode += C_TABS_1 + cFuncName + "("
    but nReturnCategory = C_TYPE_RESULT
        cCode += C_TABS_1 + "match " + cFuncName + "("
    but nReturnCategory = C_TYPE_OPTION
        cCode += C_TABS_1 + "match " + cFuncName + "("
    else
        cCode += C_TABS_1 + "let __result = " + cFuncName + "("
    ok
    
    # Add arguments (using FormatMethodArgument to handle &Struct params)
    for i = 1 to nParamCount
        if i > 1
            cCode += ", "
        ok
        cCode += FormatMethodArgument(aParams[i][2], i)
    next
    
    cCode += ")"
    
    # Return handling
    switch nReturnCategory
    on C_TYPE_VOID
        cCode += ";" + nl
    on C_TYPE_RESULT
        cCode += " {" + nl
        cInnerType = GetInnerType(cReturnType)
        nInnerCategory = GetRustTypeCategory(cInnerType)
        cCode += C_TABS_2 + "Ok(__result) => "
        cCode += GenerateReturnStatementEx(nInnerCategory, cInnerType, "__result", true)
        cCode += C_TABS_2 + 'Err(e) => ring_error!(p, &format!("{:?}", e)),' + nl
        cCode += C_TABS_1 + "}" + nl
    on C_TYPE_OPTION
        cCode += " {" + nl
        cInnerType = GetInnerType(cReturnType)
        nInnerCategory = GetRustTypeCategory(cInnerType)
        cCode += C_TABS_2 + "Some(__result) => "
        cCode += GenerateReturnStatementEx(nInnerCategory, cInnerType, "__result", true)
        cCode += C_TABS_2 + "None => ring_ret_string!(p, " + '"" ' + ")," + nl
        cCode += C_TABS_1 + "}" + nl
    other
        cCode += ";" + nl
        cCode += GenerateReturnStatement(nReturnCategory, cReturnType, "__result")
    off
    
    cCode += "});" + nl
    
    return cCode

Func GenerateParamCheck cType, nIndex
    nCategory = GetRustTypeCategory(cType)
    
    switch nCategory
    on C_TYPE_NUMBER
        return C_TABS_1 + "ring_check_number!(p, " + nIndex + ");" + nl
    on C_TYPE_BOOL
        return C_TABS_1 + "ring_check_number!(p, " + nIndex + ");" + nl
    on C_TYPE_STRING
        return C_TABS_1 + "ring_check_string!(p, " + nIndex + ");" + nl
    on C_TYPE_LIST
        return C_TABS_1 + "ring_check_list!(p, " + nIndex + ");" + nl
    on C_TYPE_POINTER
        return C_TABS_1 + "ring_check_cpointer!(p, " + nIndex + ");" + nl
    on C_TYPE_STRUCT
        return C_TABS_1 + "ring_check_cpointer!(p, " + nIndex + ");" + nl
    other
        return C_TABS_1 + "ring_check_cpointer!(p, " + nIndex + ");" + nl
    off

Func GenerateParamGet cName, cType, nIndex
    nCategory = GetRustTypeCategory(cType)
    cArgName = "__arg_" + nIndex
    
    switch nCategory
    on C_TYPE_NUMBER
        return C_TABS_1 + "let " + cArgName + " = ring_get_number!(p, " + nIndex + ") as " + cType + ";" + nl
    on C_TYPE_BOOL
        return C_TABS_1 + "let " + cArgName + " = ring_get_number!(p, " + nIndex + ") != 0.0;" + nl
    on C_TYPE_STRING
        # ring_get_string! returns &str, use directly for &str params or convert to String
        if cType = "String"
            return C_TABS_1 + "let " + cArgName + " = ring_get_string!(p, " + nIndex + ").to_string();" + nl
        else
            # For &str parameters, ring_get_string! already returns &str
            return C_TABS_1 + "let " + cArgName + " = ring_get_string!(p, " + nIndex + ");" + nl
        ok
    on C_TYPE_LIST
        return C_TABS_1 + "let " + cArgName + " = ring_get_list!(p, " + nIndex + ");" + nl
    on C_TYPE_POINTER
        cInnerType = GetPointerInnerType(cType)
        cTypeConst = upper(cInnerType) + "_TYPE"
        return C_TABS_1 + "let " + cArgName + " = ring_get_cpointer!(p, " + nIndex + ", " + cTypeConst + ");" + nl
    on C_TYPE_STRUCT
        # Struct parameter - get pointer and dereference+clone to get owned value
        cTypeConst = upper(cType) + "_TYPE"
        cCode = C_TABS_1 + "let __ptr_" + nIndex + " = ring_get_cpointer!(p, " + nIndex + ", " + cTypeConst + ");" + nl
        cCode += C_TABS_1 + "let " + cArgName + " = unsafe { (*(__ptr_" + nIndex + " as *const " + cType + ")).clone() };" + nl
        return cCode
    other
        cTypeConst = upper(cType) + "_TYPE"
        return C_TABS_1 + "let " + cArgName + " = ring_get_cpointer!(p, " + nIndex + ", " + cTypeConst + ");" + nl
    off

Func GenerateReturnStatement nCategory, cType, cVarName
    return GenerateReturnStatementEx(nCategory, cType, cVarName, false)

Func GenerateReturnStatementEx nCategory, cType, cVarName, lInMatchArm
    # lInMatchArm: true = use comma, false = use semicolon
    cEnd = ";"
    if lInMatchArm
        cEnd = ","
    ok
    
    switch nCategory
    on C_TYPE_VOID
        return ""
    on C_TYPE_NUMBER
        return C_TABS_1 + "ring_ret_number!(p, " + cVarName + " as f64)" + cEnd + nl
    on C_TYPE_BOOL
        return C_TABS_1 + "ring_ret_number!(p, if " + cVarName + " { 1.0 } else { 0.0 })" + cEnd + nl
    on C_TYPE_STRING
        return C_TABS_1 + "ring_ret_string!(p, &" + cVarName + ")" + cEnd + nl
    on C_TYPE_LIST
        # Vec<T> needs conversion to Ring list
        cInnerType = GetInnerType(cType)
        nInnerCategory = GetRustTypeCategory(cInnerType)
        cCode = ""
        if lInMatchArm
            cCode += "{" + nl
        ok
        cCode += C_TABS_1 + "let __list = ring_new_list!(p);" + nl
        cCode += C_TABS_1 + "for __item in " + cVarName + " {" + nl
        switch nInnerCategory
        on C_TYPE_NUMBER
            cCode += C_TABS_2 + "ring_list_adddouble(__list, __item as f64);" + nl
        on C_TYPE_BOOL
            cCode += C_TABS_2 + "ring_list_addint(__list, if __item { 1 } else { 0 });" + nl
        on C_TYPE_STRING
            cCode += C_TABS_2 + "ring_list_addstring(__list, format!(" + '"' + "{}\0" + '"' + ", __item).as_bytes());" + nl
        other
            # Complex inner type (likely a struct) - add as pointer
            if IsStructType(cInnerType)
                cInnerTypeConst = upper(cInnerType) + "_TYPE"
                cCode += C_TABS_2 + "let __ptr = Box::into_raw(Box::new(__item));" + nl
                cCode += C_TABS_2 + "ring_list_addcpointer(__list, __ptr as *mut std::ffi::c_void, " + cInnerTypeConst + ");" + nl
            else
                # Fallback for unknown types - use debug format
                cCode += C_TABS_2 + "// Unknown inner type: " + cInnerType + nl
                cCode += C_TABS_2 + "ring_list_addstring(__list, format!(" + '"' + "{:?}\0" + '"' + ", __item).as_bytes());" + nl
            ok
        off
        cCode += C_TABS_1 + "}" + nl
        cCode += C_TABS_1 + "ring_ret_list!(p, __list)" + cEnd + nl
        if lInMatchArm
            cCode += C_TABS_1 + "}" + nl
        ok
        return cCode
    on C_TYPE_POINTER
        cInnerType = GetPointerInnerType(cType)
        cTypeConst = upper(cInnerType) + "_TYPE"
        return C_TABS_1 + "ring_ret_cpointer!(p, " + cVarName + ", " + cTypeConst + ");" + nl
    on C_TYPE_STRUCT
        # Owned struct - box it and return as pointer
        cTypeConst = upper(cType) + "_TYPE"
        return C_TABS_1 + "ring_ret_cpointer!(p, Box::into_raw(Box::new(" + cVarName + ")), " + cTypeConst + ");" + nl
    other
        # Unknown type - try to box it
        cTypeConst = upper(cType) + "_TYPE"
        return C_TABS_1 + "ring_ret_cpointer!(p, Box::into_raw(Box::new(" + cVarName + ")), " + cTypeConst + ");" + nl
    off

Func GenerateStructWrappers aStruct, aImplMethods
    # aStruct = [name, [[field_name, field_type], ...], [attributes]]
    # aImplMethods = [[struct_name, method_name], ...] - methods to skip accessor generation for
    
    cStructName = aStruct[1]
    aFields = aStruct[2]
    
    cPrefix = $cLibPrefix
    if cPrefix != ""
        cPrefix += "_"
    ok
    cLowerName = lower(cStructName)
    cTypeConst = upper(cStructName) + "_TYPE"
    
    cCode = nl
    cCode += "// ============================================" + nl
    cCode += "// Struct: " + cStructName + nl
    cCode += "// ============================================" + nl
    cCode += 'const ' + cTypeConst + ': &[u8] = b"' + cStructName + '\0";' + nl
    cCode += nl
    
    # Constructor (new) - check for custom constructor
    aCustomCtor = GetCustomConstructor(cStructName)
    
    cCode += "// Constructor" + nl
    cCode += "ring_func!(ring_" + cPrefix + cLowerName + "_new, |p| {" + nl
    
    if len(aCustomCtor) > 0
        # Custom constructor with parameters
        aParams = aCustomCtor[2]
        nParamCount = len(aParams)
        
        cCode += C_TABS_1 + "ring_check_paracount!(p, " + nParamCount + ");" + nl
        
        # Parameter type checks
        for i = 1 to nParamCount
            cCode += GenerateParamCheck(aParams[i][2], i)
        next
        
        cCode += nl
        
        # Get parameters
        for i = 1 to nParamCount
            cCode += GenerateParamGet(aParams[i][1], aParams[i][2], i)
        next
        
        # Call custom constructor
        cCode += nl
        cCode += C_TABS_1 + "let obj = Box::new(" + cStructName + "::new("
        for i = 1 to nParamCount
            if i > 1
                cCode += ", "
            ok
            cCode += "__arg_" + i
        next
        cCode += "));" + nl
    else
        # Default constructor (no parameters)
        cCode += C_TABS_1 + "ring_check_paracount!(p, 0);" + nl
        cCode += C_TABS_1 + "let obj = Box::new(" + cStructName + "::default());" + nl
    ok
    
    cCode += C_TABS_1 + "ring_ret_cpointer!(p, Box::into_raw(obj), " + cTypeConst + ");" + nl
    cCode += "});" + nl
    cCode += nl
    
    # Destructor (delete)
    cCode += "// Destructor" + nl
    cCode += "ring_func!(ring_" + cPrefix + cLowerName + "_delete, |p| {" + nl
    cCode += C_TABS_1 + "ring_check_paracount!(p, 1);" + nl
    cCode += C_TABS_1 + "ring_check_cpointer!(p, 1);" + nl
    cCode += C_TABS_1 + "let ptr = ring_get_cpointer!(p, 1, " + cTypeConst + ");" + nl
    cCode += C_TABS_1 + "if !ptr.is_null() {" + nl
    cCode += C_TABS_2 + "unsafe { let _ = Box::from_raw(ptr as *mut " + cStructName + "); }" + nl
    cCode += C_TABS_1 + "}" + nl
    cCode += "});" + nl
    cCode += nl
    

    
    # Field accessors
    for aField in aFields
        cFieldName = aField[1]
        cFieldType = aField[2]
        nFieldCategory = GetRustTypeCategory(cFieldType)
        
        # Check if there's an impl method that conflicts with get_fieldname or set_fieldname
        lSkipGetter = false
        lSkipSetter = false
        for aImplMethod in aImplMethods
            if aImplMethod[1] = cStructName
                cMethodName = aImplMethod[2]
                if cMethodName = "get_" + cFieldName or cMethodName = cFieldName
                    lSkipGetter = true
                ok
                if cMethodName = "set_" + cFieldName
                    lSkipSetter = true
                ok
            ok
        next
        
        # Getter
        if not lSkipGetter
            cCode += "// Getter: " + cFieldName + nl
            cCode += "ring_func!(ring_" + cPrefix + cLowerName + "_get_" + cFieldName + ", |p| {" + nl
            cCode += C_TABS_1 + "ring_check_paracount!(p, 1);" + nl
            cCode += C_TABS_1 + "ring_check_cpointer!(p, 1);" + nl
            cCode += C_TABS_1 + "if let Some(obj) = ring_get_pointer!(p, 1, " + cStructName + ", " + cTypeConst + ") {" + nl
            
            switch nFieldCategory
            on C_TYPE_NUMBER
                cCode += C_TABS_2 + "ring_ret_number!(p, obj." + cFieldName + ");" + nl
            on C_TYPE_BOOL
                cCode += C_TABS_2 + "ring_ret_number!(p, if obj." + cFieldName + " { 1.0 } else { 0.0 });" + nl
            on C_TYPE_STRING
                cCode += C_TABS_2 + "ring_ret_string!(p, &obj." + cFieldName + ");" + nl
            on C_TYPE_LIST
                # Vec<T> field - return as Ring list
                cInnerType = GetInnerType(cFieldType)
                nInnerCategory = GetRustTypeCategory(cInnerType)
                cCode += C_TABS_2 + "let __list = ring_new_list!(p);" + nl
                cCode += C_TABS_2 + "for __item in &obj." + cFieldName + " {" + nl
                switch nInnerCategory
                on C_TYPE_NUMBER
                    cCode += C_TABS_3 + "ring_list_adddouble(__list, *__item as f64);" + nl
                on C_TYPE_BOOL
                    cCode += C_TABS_3 + "ring_list_addint(__list, if *__item { 1 } else { 0 });" + nl
                on C_TYPE_STRING
                    cCode += C_TABS_3 + "ring_list_addstring(__list, format!(" + '"' + "{}\0" + '"' + ", __item).as_bytes());" + nl
                other
                    if IsStructType(cInnerType)
                        cInnerTypeConst = upper(cInnerType) + "_TYPE"
                        cCode += C_TABS_3 + "let __ptr = Box::into_raw(Box::new(__item.clone()));" + nl
                        cCode += C_TABS_3 + "ring_list_addcpointer(__list, __ptr as *mut std::ffi::c_void, " + cInnerTypeConst + ");" + nl
                    else
                        cCode += C_TABS_3 + "ring_list_addstring(__list, format!(" + '"' + "{:?}\0" + '"' + ", __item).as_bytes());" + nl
                    ok
                off
                cCode += C_TABS_2 + "}" + nl
                cCode += C_TABS_2 + "ring_ret_list!(p, __list);" + nl
            on C_TYPE_OPTION
                # Option<T> field - return inner value or null
                cInnerType = GetInnerType(cFieldType)
                nInnerCategory = GetRustTypeCategory(cInnerType)
                cCode += C_TABS_2 + "match &obj." + cFieldName + " {" + nl
                cCode += C_TABS_3 + "Some(__val) => {" + nl
                switch nInnerCategory
                on C_TYPE_NUMBER
                    cCode += C_TABS_4 + "ring_ret_number!(p, *__val as f64);" + nl
                on C_TYPE_BOOL
                    cCode += C_TABS_4 + "ring_ret_number!(p, if *__val { 1.0 } else { 0.0 });" + nl
                on C_TYPE_STRING
                    cCode += C_TABS_4 + "ring_ret_string!(p, __val);" + nl
                other
                    if IsStructType(cInnerType)
                        cInnerTypeConst = upper(cInnerType) + "_TYPE"
                        cCode += C_TABS_4 + "ring_ret_cpointer!(p, Box::into_raw(Box::new(__val.clone())), " + cInnerTypeConst + ");" + nl
                    else
                        cCode += C_TABS_4 + "ring_ret_string!(p, &format!(" + '"' + "{:?}" + '"' + ", __val));" + nl
                    ok
                off
                cCode += C_TABS_3 + "}" + nl
                cCode += C_TABS_3 + 'None => ring_ret_string!(p, ""),' + nl
                cCode += C_TABS_2 + "}" + nl
            on C_TYPE_STRUCT
                # Struct field - return clone as pointer
                cFieldTypeConst = upper(cFieldType) + "_TYPE"
                cCode += C_TABS_2 + "ring_ret_cpointer!(p, Box::into_raw(Box::new(obj." + cFieldName + ".clone())), " + cFieldTypeConst + ");" + nl
            other
                # Unknown type - return 0
                cCode += C_TABS_2 + "// Unknown field type: " + cFieldType + nl
                cCode += C_TABS_2 + "ring_ret_number!(p, 0.0);" + nl
            off
            
            cCode += C_TABS_1 + "} else {" + nl
            cCode += C_TABS_2 + 'ring_error!(p, "Invalid ' + cStructName + ' pointer");' + nl
            cCode += C_TABS_1 + "}" + nl
            cCode += "});" + nl
            cCode += nl
        ok
        
        # Setter
        if not lSkipSetter
            cCode += "// Setter: " + cFieldName + nl
            cCode += "ring_func!(ring_" + cPrefix + cLowerName + "_set_" + cFieldName + ", |p| {" + nl
            cCode += C_TABS_1 + "ring_check_paracount!(p, 2);" + nl
            cCode += C_TABS_1 + "ring_check_cpointer!(p, 1);" + nl
            
            switch nFieldCategory
            on C_TYPE_NUMBER
                cCode += C_TABS_1 + "ring_check_number!(p, 2);" + nl
            on C_TYPE_BOOL
                cCode += C_TABS_1 + "ring_check_number!(p, 2);" + nl
            on C_TYPE_STRING
                cCode += C_TABS_1 + "ring_check_string!(p, 2);" + nl
            on C_TYPE_LIST
                cCode += C_TABS_1 + "ring_check_list!(p, 2);" + nl
            on C_TYPE_OPTION
                # Option can be pointer (Some) or empty string (None)
                cCode += C_TABS_1 + "// Option field - accepts pointer or empty string for None" + nl
            on C_TYPE_STRUCT
                cCode += C_TABS_1 + "ring_check_cpointer!(p, 2);" + nl
            other
                cCode += C_TABS_1 + "ring_check_cpointer!(p, 2);" + nl
            off
            
            cCode += C_TABS_1 + "if let Some(obj) = ring_get_pointer!(p, 1, " + cStructName + ", " + cTypeConst + ") {" + nl
            
            switch nFieldCategory
            on C_TYPE_NUMBER
                cCode += C_TABS_2 + "obj." + cFieldName + " = ring_get_number!(p, 2) as " + cFieldType + ";" + nl
            on C_TYPE_BOOL
                cCode += C_TABS_2 + "obj." + cFieldName + " = ring_get_number!(p, 2) != 0.0;" + nl
            on C_TYPE_STRING
                # ring_get_string! returns &str, need to convert to String for owned field
                cCode += C_TABS_2 + "obj." + cFieldName + " = ring_get_string!(p, 2).to_string();" + nl
            on C_TYPE_LIST
                # Vec<T> field - convert from Ring list
                cInnerType = GetInnerType(cFieldType)
                nInnerCategory = GetRustTypeCategory(cInnerType)
                cCode += C_TABS_2 + "let __list = ring_get_list!(p, 2);" + nl
                cCode += C_TABS_2 + "let __size = ring_list_getsize(__list);" + nl
                cCode += C_TABS_2 + "let mut __vec = Vec::new();" + nl
                cCode += C_TABS_2 + "for __i in 1..=__size {" + nl
                switch nInnerCategory
                on C_TYPE_NUMBER
                    cCode += C_TABS_3 + "__vec.push(ring_list_getdouble(__list, __i) as " + cInnerType + ");" + nl
                on C_TYPE_BOOL
                    cCode += C_TABS_3 + "__vec.push(ring_list_getint(__list, __i) != 0);" + nl
                on C_TYPE_STRING
                    cCode += C_TABS_3 + "__vec.push(ring_list_getstring(__list, __i).to_string());" + nl
                other
                    if IsStructType(cInnerType)
                        cInnerTypeConst = upper(cInnerType) + "_TYPE"
                        cCode += C_TABS_3 + "let __ptr = ring_list_getcpointer(__list, __i, " + cInnerTypeConst + ");" + nl
                        cCode += C_TABS_3 + "if !__ptr.is_null() {" + nl
                        cCode += C_TABS_4 + "__vec.push(unsafe { (*(__ptr as *const " + cInnerType + ")).clone() });" + nl
                        cCode += C_TABS_3 + "}" + nl
                    else
                        cCode += C_TABS_3 + "// Cannot convert unknown type from list" + nl
                    ok
                off
                cCode += C_TABS_2 + "}" + nl
                cCode += C_TABS_2 + "obj." + cFieldName + " = __vec;" + nl
            on C_TYPE_OPTION
                # Option<T> field - accept pointer for Some, check for None
                cInnerType = GetInnerType(cFieldType)
                nInnerCategory = GetRustTypeCategory(cInnerType)
                cCode += C_TABS_2 + "if ring_vm_api_isstring(p, 2) != 0 && ring_get_string!(p, 2).is_empty() {" + nl
                cCode += C_TABS_3 + "obj." + cFieldName + " = None;" + nl
                cCode += C_TABS_2 + "} else {" + nl
                switch nInnerCategory
                on C_TYPE_NUMBER
                    cCode += C_TABS_3 + "obj." + cFieldName + " = Some(ring_get_number!(p, 2) as " + cInnerType + ");" + nl
                on C_TYPE_BOOL
                    cCode += C_TABS_3 + "obj." + cFieldName + " = Some(ring_get_number!(p, 2) != 0.0);" + nl
                on C_TYPE_STRING
                    cCode += C_TABS_3 + "obj." + cFieldName + " = Some(ring_get_string!(p, 2).to_string());" + nl
                other
                    if IsStructType(cInnerType)
                        cInnerTypeConst = upper(cInnerType) + "_TYPE"
                        cCode += C_TABS_3 + "if let Some(val) = ring_get_pointer!(p, 2, " + cInnerType + ", " + cInnerTypeConst + ") {" + nl
                        cCode += C_TABS_4 + "obj." + cFieldName + " = Some(val.clone());" + nl
                        cCode += C_TABS_3 + "}" + nl
                    else
                        cCode += C_TABS_3 + "// Cannot set unknown Option inner type" + nl
                    ok
                off
                cCode += C_TABS_2 + "}" + nl
            on C_TYPE_STRUCT
                # Struct field - get from pointer and clone
                cFieldTypeConst = upper(cFieldType) + "_TYPE"
                cCode += C_TABS_2 + "if let Some(val) = ring_get_pointer!(p, 2, " + cFieldType + ", " + cFieldTypeConst + ") {" + nl
                cCode += C_TABS_3 + "obj." + cFieldName + " = val.clone();" + nl
                cCode += C_TABS_2 + "} else {" + nl
                cCode += C_TABS_3 + 'ring_error!(p, "Invalid ' + cFieldType + ' pointer");' + nl
                cCode += C_TABS_2 + "}" + nl
            other
                # Unknown type - no-op
                cCode += C_TABS_2 + "// Unknown field type: " + cFieldType + " - cannot set" + nl
            off
            
            cCode += C_TABS_1 + "} else {" + nl
            cCode += C_TABS_2 + 'ring_error!(p, "Invalid ' + cStructName + ' pointer");' + nl
            cCode += C_TABS_1 + "}" + nl
            cCode += "});" + nl
            cCode += nl
        ok
    next
    
    return cCode

Func GenerateImplWrappers aImpl
    # aImpl = [struct_name, [[method_name, params, return_type, is_static], ...]]
    
    cStructName = aImpl[1]
    aMethods = aImpl[2]
    
    cPrefix = $cLibPrefix
    if cPrefix != ""
        cPrefix += "_"
    ok
    cLowerName = lower(cStructName)
    cTypeConst = upper(cStructName) + "_TYPE"
    
    cCode = nl
    cCode += "// ============================================" + nl
    cCode += "// Impl: " + cStructName + nl
    cCode += "// ============================================" + nl
    
    for aMethod in aMethods
        cMethodName = aMethod[1]
        aParams = aMethod[2]
        cReturnType = aMethod[3]
        lStatic = aMethod[4]
        
        # Resolve Self to actual struct name
        if cReturnType = "Self"
            cReturnType = cStructName
        ok
        
        nParamCount = len(aParams)
        if not lStatic
            nParamCount++  # Add 1 for self pointer
        ok
        
        cCode += nl
        cCode += "// Method: " + cMethodName + nl
        cCode += "ring_func!(ring_" + cPrefix + cLowerName + "_" + cMethodName + ", |p| {" + nl
        cCode += C_TABS_1 + "ring_check_paracount!(p, " + nParamCount + ");" + nl
        
        if not lStatic
            cCode += C_TABS_1 + "ring_check_cpointer!(p, 1);" + nl
        ok
        
        # Parameter type checks
        nOffset = 0
        if not lStatic
            nOffset = 1
        ok
        
        for i = 1 to len(aParams)
            cCode += GenerateParamCheck(aParams[i][2], i + nOffset)
        next
        
        cCode += nl
        
        # Get self pointer for non-static methods
        if not lStatic
            cCode += C_TABS_1 + "if let Some(obj) = ring_get_pointer!(p, 1, " + cStructName + ", " + cTypeConst + ") {" + nl
        ok
        
        # Get parameters
        cIndent = C_TABS_1
        if not lStatic
            cIndent = C_TABS_2
        ok
        
        for i = 1 to len(aParams)
            cCode += cIndent
            cCode += substr(GenerateParamGet(aParams[i][1], aParams[i][2], i + nOffset), C_TABS_1, "")
        next
        
        # Call method
        cCode += nl
        nReturnCategory = GetRustTypeCategory(cReturnType)
        
        if nReturnCategory = C_TYPE_VOID
            if lStatic
                cCode += cIndent + cStructName + "::" + cMethodName + "("
            else
                cCode += cIndent + "obj." + cMethodName + "("
            ok
        else
            if lStatic
                cCode += cIndent + "let __result = " + cStructName + "::" + cMethodName + "("
            else
                cCode += cIndent + "let __result = obj." + cMethodName + "("
            ok
        ok
        
        # Add arguments
        for i = 1 to len(aParams)
            if i > 1
                cCode += ", "
            ok
            cParamType = aParams[i][2]
            cCode += FormatMethodArgument(cParamType, i + nOffset)
        next
        
        cCode += ");" + nl
        
        # Return handling
        if nReturnCategory != C_TYPE_VOID
            cCode += cIndent
            cCode += substr(GenerateReturnStatement(nReturnCategory, cReturnType, "__result"), C_TABS_1, "")
        ok
        
        if not lStatic
            cCode += C_TABS_1 + "} else {" + nl
            cCode += C_TABS_2 + 'ring_error!(p, "Invalid ' + cStructName + ' pointer");' + nl
            cCode += C_TABS_1 + "}" + nl
        ok
        
        cCode += "});" + nl
    next
    
    return cCode

Func GenerateConstantGetter aConst
    # aConst = [name, type, value]
    
    cName = aConst[1]
    cType = aConst[2]
    
    cPrefix = $cLibPrefix
    if cPrefix != ""
        cPrefix += "_"
    ok
    
    cCode = nl
    cCode += "// Constant: " + cName + nl
    cCode += "ring_func!(ring_" + cPrefix + "get_" + lower(cName) + ", |p| {" + nl
    cCode += C_TABS_1 + "ring_check_paracount!(p, 0);" + nl
    
    nCategory = GetRustTypeCategory(cType)
    switch nCategory
    on C_TYPE_NUMBER
        cCode += C_TABS_1 + "ring_ret_number!(p, " + cName + ");" + nl
    on C_TYPE_BOOL
        cCode += C_TABS_1 + "ring_ret_number!(p, if " + cName + " { 1.0 } else { 0.0 });" + nl
    on C_TYPE_STRING
        cCode += C_TABS_1 + "ring_ret_string!(p, " + cName + ");" + nl
    other
        cCode += C_TABS_1 + "ring_ret_number!(p, " + cName + " as f64);" + nl
    off
    
    cCode += "});" + nl
    
    return cCode

Func GetFunctionRegistration aFunc
    cFuncName = aFunc[1]
    cPrefix = $cLibPrefix
    if cPrefix != ""
        cPrefix += "_"
    ok
    cWrapperName = "ring_" + cPrefix + cFuncName
    cRingName = cPrefix + cFuncName
    return [cRingName, cWrapperName]

Func GetStructRegistrations aStruct, aImplMethods
    cStructName = aStruct[1]
    aFields = aStruct[2]
    
    cPrefix = $cLibPrefix
    if cPrefix != ""
        cPrefix += "_"
    ok
    cLowerName = lower(cStructName)
    
    aRegs = []
    
    # new, delete
    aRegs + [cPrefix + cLowerName + "_new", "ring_" + cPrefix + cLowerName + "_new"]
    aRegs + [cPrefix + cLowerName + "_delete", "ring_" + cPrefix + cLowerName + "_delete"]
    
    # Field accessors (skip if impl method exists)
    for aField in aFields
        cFieldName = aField[1]
        
        # Check for conflicts with impl methods
        lSkipGetter = false
        lSkipSetter = false
        for aImplMethod in aImplMethods
            if aImplMethod[1] = cStructName
                cMethodName = aImplMethod[2]
                if cMethodName = "get_" + cFieldName or cMethodName = cFieldName
                    lSkipGetter = true
                ok
                if cMethodName = "set_" + cFieldName
                    lSkipSetter = true
                ok
            ok
        next
        
        if not lSkipGetter
            aRegs + [cPrefix + cLowerName + "_get_" + cFieldName, "ring_" + cPrefix + cLowerName + "_get_" + cFieldName]
        ok
        if not lSkipSetter
            aRegs + [cPrefix + cLowerName + "_set_" + cFieldName, "ring_" + cPrefix + cLowerName + "_set_" + cFieldName]
        ok
    next
    
    return aRegs

Func GetImplRegistrations aImpl
    cStructName = aImpl[1]
    aMethods = aImpl[2]
    
    cPrefix = $cLibPrefix
    if cPrefix != ""
        cPrefix += "_"
    ok
    cLowerName = lower(cStructName)
    
    aRegs = []
    
    for aMethod in aMethods
        cMethodName = aMethod[1]
        aRegs + [cPrefix + cLowerName + "_" + cMethodName, "ring_" + cPrefix + cLowerName + "_" + cMethodName]
    next
    
    return aRegs

Func GetConstantRegistration aConst
    cName = aConst[1]
    cPrefix = $cLibPrefix
    if cPrefix != ""
        cPrefix += "_"
    ok
    return [cPrefix + "get_" + lower(cName), "ring_" + cPrefix + "get_" + lower(cName)]

Func GenerateLibInit aFunctionsList
    cCode = nl
    cCode += "// ============================================" + nl
    cCode += "// Library Registration" + nl
    cCode += "// ============================================" + nl
    cCode += "ring_libinit! {" + nl
    
    for i = 1 to len(aFunctionsList)
        aReg = aFunctionsList[i]
        cCode += C_TABS_1 + 'b"' + aReg[1] + '\0" => ' + aReg[2]
        if i < len(aFunctionsList)
            cCode += ","
        ok
        cCode += nl
    next
    
    cCode += "}" + nl
    
    return cCode

# ==============================================================================
# Ring Class Generation
# ==============================================================================

Func GenerateRingClasses aData
    cCode = ""
    cCode += "# Generated by parsec.ring" + nl
    cCode += "# Do not edit manually" + nl
    cCode += nl
    cCode += 'load "codegenlib.ring"' + nl
    cCode += nl
    
    cPrefix = $cLibPrefix
    if cPrefix != ""
        cPrefix += "_"
    ok
    
    # Generate classes for structs
    for aEntry in aData
        if aEntry[1] = C_INS_STRUCT
            aStruct = ParseStruct(aEntry[2])
            if aStruct[1] != ""
                cCode += GenerateRingClass(aStruct, cPrefix, aData)
            ok
        ok
    next
    
    return cCode

Func GenerateRingClass aStruct, cPrefix, aData
    cStructName = aStruct[1]
    aFields = aStruct[2]
    cLowerName = lower(cStructName)
    
    cCode = nl
    cCode += "Class " + cStructName + nl
    cCode += nl
    cCode += C_TABS_1 + "pObject" + nl
    cCode += nl
    
    # Constructor - check for custom constructor
    aCustomCtor = GetCustomConstructor(cStructName)
    
    if len(aCustomCtor) > 0
        # Custom constructor with parameters
        aParams = aCustomCtor[2]
        cCode += C_TABS_1 + "Func init"
        for i = 1 to len(aParams)
            cSafeParamName = RingSafeMethodName(aParams[i][1])
            cCode += " " + cSafeParamName
            if i < len(aParams)
                cCode += ","
            ok
        next
        cCode += nl
        cCode += C_TABS_2 + "pObject = " + cPrefix + cLowerName + "_new("
        for i = 1 to len(aParams)
            if i > 1
                cCode += ", "
            ok
            cSafeParamName = RingSafeMethodName(aParams[i][1])
            # Check if parameter is a struct type that needs unwrapping
            cParamType = aParams[i][2]
            nParamCategory = GetRustTypeCategory(cParamType)
            if nParamCategory = C_TYPE_POINTER or nParamCategory = C_TYPE_STRUCT or nParamCategory = C_TYPE_UNKNOWN
                cCode += "GetObjectPointerFromRingObject(" + cSafeParamName + ")"
            else
                cCode += cSafeParamName
            ok
        next
        cCode += ")" + nl
    else
        # Default constructor (no parameters)
        cCode += C_TABS_1 + "Func init()" + nl
        cCode += C_TABS_2 + "pObject = " + cPrefix + cLowerName + "_new()" + nl
    ok
    cCode += C_TABS_2 + "return self" + nl
    cCode += nl
    
    # Destructor
    cCode += C_TABS_1 + "Func delete" + nl
    cCode += C_TABS_2 + cPrefix + cLowerName + "_delete(pObject)" + nl
    cCode += nl
    
    # Object pointer accessor
    cCode += C_TABS_1 + "Func ObjectPointer" + nl
    cCode += C_TABS_2 + "return pObject" + nl
    cCode += nl
    
    # Field accessors
    for aField in aFields
        cFieldName = aField[1]
        cSafeFieldName = RingSafeMethodName(cFieldName)
        
        cFieldType = aField[2]
        nFieldCategory = GetRustTypeCategory(cFieldType)
        
        # Getter
        cCode += C_TABS_1 + "Func " + cSafeFieldName + nl
        cCode += C_TABS_2 + "return " + cPrefix + cLowerName + "_get_" + cFieldName + "(pObject)" + nl
        cCode += nl
        
        # Setter
        cCode += C_TABS_1 + "Func set" + Upper(Left(cFieldName, 1)) + Right(cFieldName, len(cFieldName) - 1) + " value" + nl
        if nFieldCategory = C_TYPE_POINTER or nFieldCategory = C_TYPE_STRUCT or nFieldCategory = C_TYPE_UNKNOWN
            cCode += C_TABS_2 + cPrefix + cLowerName + "_set_" + cFieldName + "(pObject, GetObjectPointerFromRingObject(value))" + nl
        else
            cCode += C_TABS_2 + cPrefix + cLowerName + "_set_" + cFieldName + "(pObject, value)" + nl
        ok
        cCode += nl
    next
    
    # Add methods from impl blocks
    for aEntry in aData
        if aEntry[1] = C_INS_IMPL
            aImpl = ParseImpl(aEntry[2])
            if aImpl[1] = cStructName
                for aMethod in aImpl[2]
                    cMethodName = aMethod[1]
                    aParams = aMethod[2]
                    aParamTypes = aMethod[2]  # Contains [name, type] pairs
                    lStatic = aMethod[4]
                    
                    if lStatic
                        loop  # Skip static methods in class
                    ok
                    
                    # Rename if it's a Ring reserved keyword
                    cRingMethodName = RingSafeMethodName(cMethodName)
                    
                    # Method
                    cCode += C_TABS_1 + "Func " + cRingMethodName
                    for i = 1 to len(aParams)
                        cCode += " P" + i
                        if i < len(aParams)
                            cCode += ","
                        ok
                    next
                    cCode += nl
                    
                    cCode += C_TABS_2 + "return " + cPrefix + cLowerName + "_" + cMethodName + "(pObject"
                    for i = 1 to len(aParams)
                        # Check if parameter is a pointer/object type that needs unwrapping
                        cParamType = ""
                        if i <= len(aParamTypes)
                            cParamType = aParamTypes[i][2]
                        ok
                        nParamCategory = GetRustTypeCategory(cParamType)
                        if nParamCategory = C_TYPE_POINTER or nParamCategory = C_TYPE_STRUCT or nParamCategory = C_TYPE_UNKNOWN
                            cCode += ", GetObjectPointerFromRingObject(P" + i + ")"
                        else
                            cCode += ", P" + i
                        ok
                    next
                    cCode += ")" + nl
                    cCode += nl
                next
            ok
        ok
    next
    
    return cCode

Func RingSafeMethodName cName
    # Rename Ring reserved keywords to safe alternatives
    cLower = lower(cName)
    if find($aRingKeywords, cLower) > 0
        switch cLower
        on "get"
            return "getValue"
        on "set" 
            return "setValue"
        on "put"
            return "putValue"
        on "give"
            return "giveValue"
        on "new"
            return "create"
        on "delete"
            return "destroy"
        on "see"
            return "show"
        on "load"
            return "loadData"
        on "call"
            return "invoke"
        on "return"
            return "getReturn"
        other
            return cName + "_"
        off
    ok
    return cName

# ==============================================================================
# Utility Functions
# ==============================================================================

Func FormatMethodArgument cType, nIndex
    # Format argument for method call, handling reference types
    cArgName = "__arg_" + nIndex
    cType = trim(cType)
    
    # Skip if it's a string type (various forms)
    if IsStringRefType(cType)
        return cArgName
    ok
    
    # Skip slices (&[T], &mut [T])
    if substr(cType, "[") > 0
        return cArgName
    ok
    
    # Check if it's a reference to a struct (e.g., &Point, &mut Point)
    if left(cType, 5) = "&mut "
        cInnerType = trim(substr(cType, 6))
        if IsStructType(cInnerType)
            return "unsafe { &mut *(" + cArgName + " as *mut " + cInnerType + ") }"
        ok
    elseif left(cType, 1) = "&"
        cInnerType = trim(substr(cType, 2))
        if IsStructType(cInnerType)
            return "unsafe { &*(" + cArgName + " as *const " + cInnerType + ") }"
        ok
    ok
    
    return cArgName

Func IsStringRefType cType
    # Check if type is any form of string reference
    cType = trim(cType)
    if cType = "&str" return true ok
    if cType = "&mut str" return true ok
    if substr(cType, "str") > 0 and left(cType, 1) = "&"
        return true
    ok
    if substr(cType, "String") > 0
        return true
    ok
    return false

Func IsStructType cType
    # Check if type is likely a struct (not a primitive, string, or slice)
    cType = trim(cType)
    
    # Not a struct if it's a primitive number
    for cNumType in $aNumberTypes
        if cType = cNumType
            return false
        ok
    next
    
    # Not a struct if it's a string type
    if cType = "str" or cType = "String"
        return false
    ok
    
    # Not a struct if it's bool
    if cType = "bool"
        return false
    ok
    
    # Not a struct if it starts with [ (slice inner)
    if left(cType, 1) = "["
        return false
    ok
    
    # Not a struct if it contains lifetime (e.g., 'static, 'a)
    if substr(cType, "'") > 0
        return false
    ok
    
    # Looks like a struct name (starts with uppercase A-Z)
    if isupper(left(cType, 1))
        return true
    ok
    
    return false

Func GetCustomConstructor cStructName
    # Returns custom constructor for struct if exists, empty list otherwise
    # Format: [struct_name, [[param_name, param_type], ...], return_type]
    for aCtor in $aCustomConstructors
        if aCtor[1] = cStructName
            return aCtor
        ok
    next
    return []

Func WriteFile cFileName, cCode
    ? "Writing: " + cFileName + " (" + len(cCode) + " bytes)"
    write(cFileName, cCode)
