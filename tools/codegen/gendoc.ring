# gendoc.ring - Generate Markdown API documentation from .rf files
#
# Usage: ring gendoc.ring input.rf [output.md]
#
# Parses .rf configuration files and generates Markdown documentation.
# Supports: <meta>, <code>, <functions>, <struct>, <impl>, <constants>, <register>
#
# Author: Youssef Saeed (ysdragon)

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

if len(sysargv) < 3
    ? "Usage: ring gendoc.ring input.rf [output.md]"
    ? ""
    ? "Generates Markdown API documentation from .rf configuration files."
    return
ok

cInputFile = sysargv[3]
cOutputFile = ""
if len(sysargv) >= 4
    cOutputFile = sysargv[4]
ok

if not fexists(cInputFile)
    ? "Error: File not found: " + cInputFile
    return
ok

cFileContent = read(cInputFile)
aLines = str2list(cFileContent)

$cLibPrefix = ""
$cCrateName = ""
$aFunctions = []
$aStructs = []
$aImpls = []
$aConstants = []

ParseFile(aLines)
cMarkdown = GenerateMarkdown()

if cOutputFile != ""
    write(cOutputFile, cMarkdown)
    ? "Generated: " + cOutputFile
else
    ? cMarkdown
ok

func ParseFile aLines
    nFlag = 0
    C_NONE = 0
    C_META = 1
    C_CODE = 2
    C_FUNCTIONS = 3
    C_STRUCT = 4
    C_IMPL = 5
    C_CONSTANTS = 6
    C_COMMENT = 7
    C_REGISTER = 8
    
    cStructData = ""
    cImplData = ""
    cLastComment = ""
    cLastSigComment = ""   # For "// name(params) - description" style comments
    aRegisteredFuncs = []  # Store ring_func! functions found in code
    
    for cLine in aLines
        cTrimmed = trim(cLine)
        
        if cTrimmed = "" and nFlag != C_CODE
            loop
        ok
        
        if left(cTrimmed, 1) = "#" and nFlag != C_CODE
            if left(cTrimmed, 2) = "# "
                cLastComment = substr(cTrimmed, 3)
            ok
            loop
        ok
        
        switch lower(cTrimmed)
        on "<meta>"
            nFlag = C_META
            loop
        on "</meta>"
            nFlag = C_NONE
            loop
        on "<code>"
            nFlag = C_CODE
            loop
        on "</code>"
            nFlag = C_NONE
            loop
        on "<functions>"
            nFlag = C_FUNCTIONS
            loop
        on "</functions>"
            nFlag = C_NONE
            loop
        on "<struct>"
            nFlag = C_STRUCT
            cStructData = ""
            loop
        on "</struct>"
            if cStructData != ""
                $aStructs + [cStructData, cLastComment]
                cLastComment = ""
            ok
            nFlag = C_NONE
            loop
        on "<impl>"
            nFlag = C_IMPL
            cImplData = ""
            loop
        on "</impl>"
            if cImplData != ""
                $aImpls + [cImplData, cLastComment]
                cLastComment = ""
            ok
            nFlag = C_NONE
            loop
        on "<constants>"
            nFlag = C_CONSTANTS
            loop
        on "</constants>"
            nFlag = C_NONE
            loop
        on "<comment>"
            nFlag = C_COMMENT
            loop
        on "</comment>"
            nFlag = C_NONE
            loop
        on "<register>"
            nFlag = C_REGISTER
            loop
        on "</register>"
            nFlag = C_NONE
            loop
        off
        
        switch nFlag
        on C_META
            ParseMeta(cTrimmed)
        on C_CODE
            # Detect pub fn for auto-wrapping
            if left(cTrimmed, 7) = "pub fn "
                lIsMethod = substr(cTrimmed, "&self") > 0 or 
                            substr(cTrimmed, "&mut self") > 0 or
                            substr(cTrimmed, "-> Self") > 0
                if not lIsMethod
                    nBracePos = substr(cTrimmed, "{")
                    if nBracePos > 0
                        cSig = trim(left(cTrimmed, nBracePos - 1))
                        cSig = trim(substr(cSig, 5))
                        $aFunctions + [cSig, cLastComment]
                        cLastComment = ""
                    ok
                ok
            ok
            # Detect ring_func! for manual functions
            if substr(cTrimmed, "ring_func!") > 0
                # Extract function name from ring_func!(ring_PREFIX_NAME, ...)
                nStart = substr(cTrimmed, "ring_func!(")
                if nStart > 0
                    cRest = substr(cTrimmed, nStart + 11)
                    nComma = substr(cRest, ",")
                    if nComma > 0
                        cRustFuncName = trim(left(cRest, nComma - 1))
                        # Store with the preceding signature comment
                        aRegisteredFuncs + [cRustFuncName, cLastSigComment, cLastComment]
                        cLastSigComment = ""
                        cLastComment = ""
                    ok
                ok
            ok
            # Capture /// doc comments
            if left(cTrimmed, 4) = "/// "
                cLastComment = substr(cTrimmed, 5)
            ok
            # Capture // signature comments like "// encode(list [, pretty]) - description"
            # Skip separator lines like "// ========"
            if left(cTrimmed, 3) = "// "
                cCommentContent = substr(cTrimmed, 4)
                # Check if it's a separator line - don't reset sig comment for these
                lIsSeparator = (substr(cCommentContent, "===") > 0 or 
                               substr(cCommentContent, "---") > 0 or
                               substr(cCommentContent, "***") > 0)
                if not lIsSeparator
                    # Check if it looks like a signature (has parens)
                    if substr(cCommentContent, "(") > 0 and substr(cCommentContent, ")") > 0
                        cLastSigComment = cCommentContent
                    ok
                ok
            ok
        on C_FUNCTIONS
            if left(lower(cTrimmed), 3) = "fn "
                $aFunctions + [cTrimmed, cLastComment]
                cLastComment = ""
            ok
        on C_STRUCT
            cStructData += cTrimmed + nl
        on C_IMPL
            cImplData += cTrimmed + nl
        on C_CONSTANTS
            if cTrimmed != ""
                $aConstants + [cTrimmed, cLastComment]
                cLastComment = ""
            ok
        on C_REGISTER
            if cTrimmed != "" and left(cTrimmed, 1) != "#"
                # Parse register line: "name" or "ring_name => rust_name"
                nArrow = substr(cTrimmed, "=>")
                if nArrow > 0
                    cRingName = trim(left(cTrimmed, nArrow - 1))
                    cRustName = trim(substr(cTrimmed, nArrow + 2))
                else
                    cRingName = cTrimmed
                    # Default rust name: ring_PREFIX_name
                    cRustName = "ring_" + $cLibPrefix + "_" + cRingName
                ok
                # Look up in aRegisteredFuncs for signature/description
                cSig = "fn " + cRingName + "(...)"
                cDesc = "Registered function"
                nFuncCount = len(aRegisteredFuncs)
                for i = 1 to nFuncCount
                    aFunc = aRegisteredFuncs[i]
                    if aFunc[1] = cRustName
                        # Found it! Parse the signature comment
                        if aFunc[2] != ""
                            # Parse "name(params) - description" or "name(params)"
                            cSigComment = aFunc[2]
                            nDash = substr(cSigComment, " - ")
                            if nDash > 0
                                cSigPart = trim(left(cSigComment, nDash - 1))
                                cDesc = trim(substr(cSigComment, nDash + 3))
                            else
                                cSigPart = cSigComment
                            ok
                            cSig = "fn " + cSigPart
                        ok
                        # Use doc comment if no sig comment
                        if aFunc[3] != "" and cDesc = "Registered function"
                            cDesc = aFunc[3]
                        ok
                        exit
                    ok
                next
                $aFunctions + [cSig, cDesc]
            ok
        off
    next

func ParseMeta cLine
    # Use parser
    ParseMetaLine(cLine)

func GenerateMarkdown
    cMD = ""
    cPrefix = $cLibPrefix
    if cPrefix != ""
        cPrefix += "_"
    ok
    
    cMD += "# " + upper(left($cLibPrefix, 1)) + substr($cLibPrefix, 2) + " API Reference" + nl
    cMD += nl
    
    if $cCrateName != ""
        cMD += "> Wraps: `" + $cCrateName + "`" + nl
        cMD += nl
    ok
    
    cMD += "**Function prefix:** `" + cPrefix + "`" + nl
    cMD += nl
    
    if len($aFunctions) > 0
        cMD += "## Functions" + nl
        cMD += nl
        cMD += "| Function | Description |" + nl
        cMD += "|----------|-------------|" + nl
        
        # Deduplicate functions by name
        aSeenFuncs = []
        for aFunc in $aFunctions
            cSig = aFunc[1]
            cDesc = aFunc[2]
            aInfo = ParseFunctionSig(cSig)
            cName = aInfo[1]
            cParams = aInfo[2]
            cReturn = aInfo[3]
            
            # Skip if already seen
            lSeen = false
            for cSeenName in aSeenFuncs
                if cSeenName = cName
                    lSeen = true
                    exit
                ok
            next
            if lSeen loop ok
            aSeenFuncs + cName
            
            cRingName = "`" + cPrefix + cName + "(" + FormatParams(cParams) + ")`"
            if cReturn != "" and cReturn != "()"
                cRingName += " → " + FormatType(cReturn)
            ok
            
            cMD += "| " + cRingName + " | " + cDesc + " |" + nl
        next
        cMD += nl
    ok
    
    if len($aStructs) > 0
        cQ = char(34)  # Double quote character
        cMD += "## Classes" + nl
        cMD += nl
        cMD += "Classes are generated Ring wrappers around the underlying structs." + nl
        cMD += nl
        cMD += "```ring" + nl
        cMD += "load " + cQ + $cLibPrefix + "_classes.ring" + cQ + nl
        cMD += "```" + nl
        cMD += nl
        
        for aStruct in $aStructs
            cData = aStruct[1]
            cDesc = aStruct[2]
            aInfo = ParseStruct(cData)
            cStructName = aInfo[1]
            aFields = aInfo[2]
            
            cMD += "### " + cStructName + " Class" + nl
            cMD += nl
            if cDesc != ""
                # Replace "struct" with "class" in description for documentation
                cDesc = substr(cDesc, " structs ", " classes ")
                cDesc = substr(cDesc, " struct ", " class ")
                cDesc = substr(cDesc, "Struct ", "Class ")
                cMD += cDesc + nl
                cMD += nl
            ok
            
            cLower = lower(cStructName)
            
            aImplMethods = GetImplMethods(cStructName)
            lHasCustomNew = false
            aNewParams = []
            cNewDesc = ""
            for aMethod in aImplMethods
                if aMethod[1] = "new"
                    lHasCustomNew = true
                    aNewParams = aMethod[2]
                    cNewDesc = aMethod[4]
                ok
            next
            
            # Constructor
            cMD += "**Constructor**" + nl
            cMD += nl
            cMD += "```ring" + nl
            if lHasCustomNew
                cMD += "obj = new " + cStructName + "(" + FormatParams(aNewParams) + ")" + nl
            else
                cMD += "obj = new " + cStructName + "()" + nl
            ok
            cMD += "```" + nl
            cMD += nl
            
            # Properties (from fields)
            if len(aFields) > 0
                cMD += "**Properties**" + nl
                cMD += nl
                cMD += "| Property | Type | Getter | Setter |" + nl
                cMD += "|----------|------|--------|--------|" + nl
                
                for aField in aFields
                    cFieldName = aField[1]
                    cFieldType = aField[2]
                    # Ring class style: obj.fieldName() and obj.setFieldName(value)
                    cGetter = "`obj." + cFieldName + "()`"
                    cSetterName = "set" + upper(left(cFieldName, 1)) + substr(cFieldName, 2)
                    cSetter = "`obj." + cSetterName + "(value)`"
                    cMD += "| " + cFieldName + " | " + FormatType(cFieldType) + " | " + cGetter + " | " + cSetter + " |" + nl
                next
                cMD += nl
            ok
            
            # Methods (excluding new, getters/setters already shown in properties)
            # Separate instance methods from static methods
            aMethods = []
            aStaticMethods = []
            for aMethod in aImplMethods
                cMethodName = aMethod[1]
                lIsStatic = aMethod[5]  # 5th element is static flag
                # Skip constructor
                if cMethodName = "new" loop ok
                # Skip getters that match field names (already in properties)
                lIsFieldGetter = false
                lIsFieldSetter = false
                for aField in aFields
                    if cMethodName = "get_" + aField[1]
                        lIsFieldGetter = true
                        exit
                    ok
                    if cMethodName = "set_" + aField[1]
                        lIsFieldSetter = true
                        exit
                    ok
                next
                if not lIsFieldGetter and not lIsFieldSetter
                    if lIsStatic
                        aStaticMethods + aMethod
                    else
                        aMethods + aMethod
                    ok
                ok
            next
            
            if len(aMethods) > 0
                cMD += "**Methods**" + nl
                cMD += nl
                cMD += "| Method | Description |" + nl
                cMD += "|--------|-------------|" + nl
                
                for aMethod in aMethods
                    cMethodName = aMethod[1]
                    aParams = aMethod[2]
                    cReturn = aMethod[3]
                    cMethodDesc = aMethod[4]
                    
                    # Ring class style: obj.methodName(params)
                    cCall = "`obj." + cMethodName + "("
                    if len(aParams) > 0
                        cCall += FormatParams(aParams)
                    ok
                    cCall += ")`"
                    if cReturn != "" and cReturn != "()" and cReturn != "Self"
                        cCall += " → " + FormatType(cReturn)
                    ok
                    
                    cMD += "| " + cCall + " | " + cMethodDesc + " |" + nl
                next
                cMD += nl
            ok
            
            if len(aStaticMethods) > 0
                cMD += "**Static Methods**" + nl
                cMD += nl
                cMD += "| Method | Description |" + nl
                cMD += "|--------|-------------|" + nl
                
                for aMethod in aStaticMethods
                    cMethodName = aMethod[1]
                    aParams = aMethod[2]
                    cReturn = aMethod[3]
                    cMethodDesc = aMethod[4]
                    
                    # Static style: prefix_structname_method(params)
                    cCall = "`" + cPrefix + cLower + "_" + cMethodName + "("
                    if len(aParams) > 0
                        cCall += FormatParams(aParams)
                    ok
                    cCall += ")`"
                    if cReturn != "" and cReturn != "()" and cReturn != "Self"
                        cCall += " → " + FormatType(cReturn)
                    ok
                    
                    cMD += "| " + cCall + " | " + cMethodDesc + " |" + nl
                next
                cMD += nl
            ok
            
            # Destructor
            cMD += "**Destructor**" + nl
            cMD += nl
            cMD += "```ring" + nl
            cMD += "obj.delete()" + nl
            cMD += "```" + nl
            cMD += nl
            
            # Example usage
            cMD += "**Example**" + nl
            cMD += nl
            cMD += "```ring" + nl
            cMD += "load " + cQ + $cLibPrefix + "_classes.ring" + cQ + nl
            cMD += nl
            if lHasCustomNew and len(aNewParams) > 0
                cExampleArgs = ""
                for i = 1 to len(aNewParams)
                    aParam = aNewParams[i]
                    cParamType = aParam[2]
                    if i > 1 cExampleArgs += ", " ok
                    if cParamType = "String" or cParamType = "&str"
                        cExampleArgs += cQ + "example" + cQ
                    elseif substr(cParamType, "i") > 0 or substr(cParamType, "u") > 0
                        cExampleArgs += "0"
                    elseif substr(cParamType, "f") > 0
                        cExampleArgs += "0.0"
                    elseif cParamType = "bool"
                        cExampleArgs += "true"
                    else
                        cExampleArgs += "..."
                    ok
                next
                cMD += "obj = new " + cStructName + "(" + cExampleArgs + ")" + nl
            else
                cMD += "obj = new " + cStructName + "()" + nl
            ok
            if len(aFields) > 0
                cField = aFields[1]
                cMD += "? obj." + cField[1] + "()    # Get " + cField[1] + nl
            ok
            if len(aMethods) > 0
                aMethod = aMethods[1]
                cMethodName = aMethod[1]
                aParams = aMethod[2]
                cReturn = aMethod[3]
                cCallExample = "obj." + cMethodName + "("
                for i = 1 to len(aParams)
                    aParam = aParams[i]
                    cParamType = aParam[2]
                    if i > 1 cCallExample += ", " ok
                    if cParamType = "String" or cParamType = "&str"
                        cCallExample += cQ + "test" + cQ
                    elseif substr(cParamType, "i") > 0 or substr(cParamType, "u") > 0
                        cCallExample += "0"
                    elseif substr(cParamType, "f") > 0
                        cCallExample += "0.0"
                    elseif cParamType = "bool"
                        cCallExample += "true"
                    else
                        cCallExample += "..."
                    ok
                next
                cCallExample += ")"
                if cReturn != "" and cReturn != "()" and cReturn != "Self"
                    cMD += "? " + cCallExample + nl
                else
                    cMD += cCallExample + nl
                ok
            ok
            cMD += "obj.delete()" + nl
            cMD += "```" + nl
            cMD += nl
        next
    ok
    
    if len($aConstants) > 0
        cMD += "## Constants" + nl
        cMD += nl
        cMD += "| Constant | Type | Getter |" + nl
        cMD += "|----------|------|--------|" + nl
        
        for aConst in $aConstants
            cConstLine = aConst[1]
            nColonPos = substr(cConstLine, ":")
            if nColonPos > 0
                cConstName = trim(left(cConstLine, nColonPos - 1))
                cConstType = trim(substr(cConstLine, nColonPos + 1))
                cGetter = "`" + cPrefix + "get_" + lower(cConstName) + "()`"
                cMD += "| " + cConstName + " | " + FormatType(cConstType) + " | " + cGetter + " |" + nl
            ok
        next
        cMD += nl
    ok
    
    cMD += "---" + nl
    cMD += "*Generated by gendoc.ring*" + nl
    
    return cMD

func ParseFunctionSig cSig
    # Use parser
    return ParseFuncSignature(cSig)

func ParseParams cParamsStr
    # Use parser
    return ParseFuncParams(cParamsStr)

func ParseStruct cData
    # Use parser
    return ParseStructDef(cData)

func GetImplMethods cStructName
    aMethods = []
    
    for aImpl in $aImpls
        cData = aImpl[1]
        aBasic = ParseImplDef(cData)
        cImplName = aBasic[1]
        
        if cImplName != cStructName loop ok
        
        # Parse impl data line by line to capture /// comments
        aLines = str2list(cData)
        cLastDocComment = ""
        
        for cLine in aLines
            cLine = trim(cLine)
            
            # Capture /// doc comments
            if left(cLine, 4) = "/// "
                cLastDocComment = substr(cLine, 5)
                loop
            ok
            
            # Match method line with parsed methods
            if left(cLine, 7) = "pub fn "
                cSig = trim(substr(cLine, 5))
                aInfo = ParseFuncSignature(cSig)
                cMethodName = aInfo[1]
                aParams = aInfo[2]
                cReturn = aInfo[3]
                
                # Check if static (no &self or &mut self)
                lIsStatic = true
                aFilteredParams = []
                for aParam in aParams
                    cParamName = aParam[1]
                    if cParamName = "&self" or cParamName = "&mut self" or cParamName = "self"
                        lIsStatic = false
                    else
                        aFilteredParams + aParam
                    ok
                next
                
                aMethods + [cMethodName, aFilteredParams, cReturn, cLastDocComment, lIsStatic]
                cLastDocComment = ""
            ok
        next
    next
    
    return aMethods

func FormatParams aParams
    # Use formatter
    return FormatParamList(aParams)

func FormatType cType
    # Use formatter
    return FormatRustType(cType)
