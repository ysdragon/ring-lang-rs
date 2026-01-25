use libc::{c_char, c_double, c_int, c_uint, c_void};

use crate::{RingFunc, RingState};

pub type RingString = *mut String;
pub type RingList = *mut List;
pub type RingItem = *mut Item;
pub type RingVM = *mut VM;
pub type RingByteCode = *mut ByteCode;
pub type RingCFunction = *mut CFunction;

pub const RING_VM_STACK_SIZE: usize = 1004;
pub const RING_VM_BC_ITEMS_COUNT: usize = 2;
pub const RING_VM_CUSTOMMUTEX_COUNT: usize = 5;
pub const RING_FALSE: c_int = 0;
pub const RING_TRUE: c_int = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub union Register {
    pub iNumber: c_int,
    pub pPointer: *mut c_void,
    pub pString: *const c_char,
    pub uiNumber: c_uint,
    pub dNumber: c_double,
}

#[repr(C)]
pub struct ByteCode {
    pub bitfields: u64,
    pub aReg: [Register; RING_VM_BC_ITEMS_COUNT],
}

#[repr(C)]
pub struct CFunction {
    pub cName: *const c_char,
    pub pFunc: Option<extern "C" fn(*mut c_void)>,
    pub pNext: *mut CFunction,
}

#[repr(C)]
pub struct FuncCall {
    pub cName: *const c_char,
    pub cFileName: *const c_char,
    pub cNewFileName: *const c_char,
    pub pTempMem: RingList,
    pub pFunc: Option<extern "C" fn(*mut c_void)>,
    pub pVMState: *mut c_void,
    pub nPC: c_uint,
    pub nSP: c_uint,
    pub nLineNumber: c_uint,
    pub nCallerPC: c_uint,
    pub nListStart: c_uint,
    pub nForStep: c_uint,
    pub nExitMark: c_uint,
    pub nLoopMark: c_uint,
    pub nCurrentGlobalScope: c_uint,
    pub nActiveScopeID: c_uint,
    pub nNestedLists: c_uint,
    pub nParaCount: c_uint,
    pub bitfields: c_uint,
}

#[repr(C)]
pub struct VM {
    pub pRingState: *mut c_void,
    pub pCode: RingList,
    pub pFunctionsMap: RingList,
    pub pClassesMap: RingList,
    pub pPackagesMap: RingList,
    pub pTempMem: RingList,
    pub pNestedLists: RingList,
    pub pPCBlockFlag: RingList,
    pub pExitMark: RingList,
    pub pLoopMark: RingList,
    pub pTry: RingList,
    pub pScopeNewObj: RingList,
    pub pObjState: RingList,
    pub pBraceObject: RingList,
    pub pBraceObjects: RingList,
    pub pActiveMem: RingList,
    pub pActivePackage: RingList,
    pub pSetProperty: RingList,
    pub pForStep: RingList,
    pub pBeforeObjState: RingList,
    pub pCLibraries: RingList,
    pub pTraceData: RingList,
    pub pGlobalScopes: RingList,
    pub pActiveGlobalScopes: RingList,
    pub pDeleteLater: RingList,
    pub pDefinedGlobals: RingList,
    pub pTrackedVariables: RingList,
    pub pLiterals: RingList,
    pub pPackageName: RingString,
    pub pTrace: RingString,
    pub pByteCode: *mut ByteCode,
    pub pByteCodeIR: *mut ByteCode,
    pub cFileName: *const c_char,
    pub cPrevFileName: *const c_char,
    pub cFileNameInClassRegion: *const c_char,
    pub pGetSetObject: *mut c_void,
    pub pAssignment: *mut c_void,
    pub pFuncMutexCreate: Option<extern "C" fn() -> *mut c_void>,
    pub pFuncMutexDestroy: Option<extern "C" fn(*mut c_void)>,
    pub pFuncMutexLock: Option<extern "C" fn(*mut c_void)>,
    pub pFuncMutexUnlock: Option<extern "C" fn(*mut c_void)>,
    pub pMutex: *mut c_void,
    pub pCFunction: *mut CFunction,
    pub nCurrentGlobalScope: c_uint,
    pub nOPCode: c_uint,
    pub nSP: c_uint,
    pub nLineNumber: c_uint,
    pub nListStart: c_uint,
    pub nBlockCounter: c_uint,
    pub nFuncSP: c_uint,
    pub nCurrentFuncCall: c_uint,
    pub nCurrentScope: c_uint,
    pub nVarScope: c_uint,
    pub nScopeID: c_uint,
    pub nActiveScopeID: c_uint,
    pub nLoadAddressScope: c_uint,
    pub nEvalReallocationSize: c_uint,
    pub nCFuncParaCount: c_uint,
    pub nCFuncSP: c_uint,
    pub nEvalReturnPC: c_uint,
    pub nPC: c_uint,
    pub nPausePC: c_uint,
    pub nArgCacheCount: c_uint,
    pub bitfields: [c_uint; 4],
    pub aStack: [Item; RING_VM_STACK_SIZE],
    pub aFuncCall: [FuncCall; RING_VM_STACK_SIZE],
    pub aScopes: [List; RING_VM_STACK_SIZE],
    pub aArgCache: [*mut List; RING_VM_STACK_SIZE],
    pub aCustomMutex: [*mut c_void; RING_VM_CUSTOMMUTEX_COUNT],
}

#[repr(C)]
pub struct String {
    pub cStr: *mut c_char,
    pub nSize: c_uint,
    pub nCapacity: c_uint,
    pub cStrArray: [c_char; 32],
}

#[repr(C)]
pub struct List {
    pub pFirst: *mut c_void,
    pub pLast: *mut c_void,
    pub nSize: c_uint,
    pub nNextItem: c_uint,
    pub pLastItem: *mut c_void,
    pub pItemsArray: *mut *mut Item,
    pub pHashTable: *mut c_void,
    pub pBlocks: *mut c_void,
    pub vGC: [u8; 24],
}

#[repr(C)]
pub union ItemData {
    pub pString: *mut String,
    pub dNumber: c_double,
    pub iNumber: c_int,
    pub pPointer: *mut c_void,
    pub pList: *mut List,
    pub pFunc: Option<extern "C" fn(*mut c_void)>,
    pub fNumber: f32,
}

#[repr(C)]
pub struct Item {
    pub data: ItemData,
    pub flags: c_uint,
    pub pGCFreeFunc: Option<extern "C" fn(*mut c_void, *mut c_void)>,
}

impl Item {
    #[inline]
    pub fn nType(&self) -> c_uint {
        self.flags & 0x7
    }

    #[inline]
    pub fn nNumberFlag(&self) -> c_uint {
        (self.flags >> 3) & 0x3
    }

    #[inline]
    pub fn nObjectType(&self) -> c_uint {
        (self.flags >> 5) & 0x3
    }

    #[inline]
    pub fn lAssignment(&self) -> bool {
        (self.flags >> 7) & 0x1 != 0
    }
}

pub const ITEMTYPE_NOTHING: c_uint = 0;
pub const ITEMTYPE_STRING: c_uint = 1;
pub const ITEMTYPE_NUMBER: c_uint = 2;
pub const ITEMTYPE_POINTER: c_uint = 3;
pub const ITEMTYPE_LIST: c_uint = 4;
pub const ITEMTYPE_FUNCPOINTER: c_uint = 5;

pub const ITEM_NUMBERFLAG_NOTHING: c_uint = 0;
pub const ITEM_NUMBERFLAG_INT: c_uint = 1;
pub const ITEM_NUMBERFLAG_DOUBLE: c_uint = 2;

#[inline]
pub unsafe fn ring_list_getsize(pList: RingList) -> c_uint {
    unsafe { (*pList).nSize }
}

#[inline]
pub unsafe fn ring_list_getint(pList: RingList, nIndex: c_uint) -> c_int {
    unsafe {
        let item = ring_list_getitem(pList, nIndex);
        (*item).data.iNumber
    }
}

#[inline]
pub unsafe fn ring_list_getdouble(pList: RingList, nIndex: c_uint) -> c_double {
    unsafe {
        let item = ring_list_getitem(pList, nIndex);
        (*item).data.dNumber
    }
}

#[inline]
pub unsafe fn ring_list_getpointer(pList: RingList, nIndex: c_uint) -> *mut c_void {
    unsafe {
        let item = ring_list_getitem(pList, nIndex);
        (*item).data.pPointer
    }
}

#[inline]
pub unsafe fn ring_list_getstring(pList: RingList, nIndex: c_uint) -> *const c_char {
    unsafe {
        let item = ring_list_getitem(pList, nIndex);
        let pString = (*item).data.pString;
        (*pString).cStr
    }
}

#[inline]
pub unsafe fn ring_list_getstringsize(pList: RingList, nIndex: c_uint) -> c_uint {
    unsafe {
        let item = ring_list_getitem(pList, nIndex);
        let pString = (*item).data.pString;
        (*pString).nSize
    }
}

#[link(name = "ring")]
unsafe extern "C" {
    pub fn ring_vm_funcregister2(pRingState: RingState, cStr: *const c_char, pFunc: RingFunc);

    pub fn ring_vm_api_paracount(pPointer: *mut c_void) -> c_int;
    pub fn ring_vm_api_isstring(pPointer: *mut c_void, nPara: c_int) -> c_int;
    pub fn ring_vm_api_isnumber(pPointer: *mut c_void, nPara: c_int) -> c_int;
    pub fn ring_vm_api_ispointer(pPointer: *mut c_void, nPara: c_int) -> c_int;
    pub fn ring_vm_api_isptr(pPointer: *mut c_void, nPara: c_int) -> c_int;
    pub fn ring_vm_api_iscpointer(pPointer: *mut c_void, nPara: c_int) -> c_int;
    pub fn ring_vm_api_islist(pPointer: *mut c_void, nPara: c_int) -> c_int;
    pub fn ring_vm_api_islistornull(pPointer: *mut c_void, nPara: c_int) -> c_int;
    pub fn ring_vm_api_isobject(pPointer: *mut c_void, nPara: c_int) -> c_int;
    pub fn ring_vm_api_iscpointerlist(pPointer: *mut c_void, pList: RingList) -> c_int;

    pub fn ring_vm_api_getstring(pPointer: *mut c_void, nPara: c_int) -> *const c_char;
    pub fn ring_vm_api_getstringsize(pPointer: *mut c_void, nPara: c_int) -> c_uint;
    pub fn ring_vm_api_getstringraw(pPointer: *mut c_void) -> RingString;
    pub fn ring_vm_api_getnumber(pPointer: *mut c_void, nPara: c_int) -> c_double;
    pub fn ring_vm_api_getpointer(pPointer: *mut c_void, nPara: c_int) -> *mut c_void;
    pub fn ring_vm_api_getpointertype(pPointer: *mut c_void, nPara: c_int) -> c_int;
    pub fn ring_vm_api_getcpointer(
        pPointer: *mut c_void,
        nPara: c_int,
        cType: *const c_char,
    ) -> *mut c_void;
    pub fn ring_vm_api_getcpointer2pointer(
        pPointer: *mut c_void,
        nPara: c_int,
        cType: *const c_char,
    ) -> *mut c_void;
    pub fn ring_vm_api_getlist(pPointer: *mut c_void, nPara: c_int) -> RingList;

    pub fn ring_vm_api_setptr(pPointer: *mut c_void, nPara: c_int, pPtr: *mut c_void, nType: c_int);

    pub fn ring_vm_api_retnumber(pPointer: *mut c_void, nNumber: c_double);
    pub fn ring_vm_api_retstring(pPointer: *mut c_void, cStr: *const c_char);
    pub fn ring_vm_api_retstring2(pPointer: *mut c_void, cStr: *const c_char, nLen: c_uint);
    pub fn ring_vm_api_retstringsize(pPointer: *mut c_void, nSize: c_uint);
    pub fn ring_vm_api_retcpointer(
        pPointer: *mut c_void,
        pGeneral: *mut c_void,
        cType: *const c_char,
    );
    pub fn ring_vm_api_retcpointer2(
        pPointer: *mut c_void,
        pGeneral: *mut c_void,
        cType: *const c_char,
        pFreeFunc: Option<extern "C" fn(*mut c_void, *mut c_void)>,
    );
    pub fn ring_vm_api_retlist(pPointer: *mut c_void, pList: RingList);
    pub fn ring_vm_api_retlist2(pPointer: *mut c_void, pList: RingList, nRef: c_int);

    pub fn ring_vm_api_newlist(pPointer: *mut c_void) -> RingList;
    pub fn ring_vm_api_newlistusingblocks(
        pPointer: *mut c_void,
        nSize: c_uint,
        nSize2: c_uint,
    ) -> RingList;

    pub fn ring_vm_api_setcpointernull(pPointer: *mut c_void, nPara: c_int);
    pub fn ring_vm_api_varptr(
        pPointer: *mut c_void,
        cStr: *const c_char,
        cStr2: *const c_char,
    ) -> *mut c_void;
    pub fn ring_vm_api_varvalue(pPointer: *mut c_void, cStr: *const c_char, nType: c_int);
    pub fn ring_vm_api_intvalue(pPointer: *mut c_void, cStr: *const c_char);
    pub fn ring_vm_api_floatvalue(pPointer: *mut c_void, cStr: *const c_char);
    pub fn ring_vm_api_ignorecpointertypecheck(pPointer: *mut c_void);

    pub fn ring_vm_api_callerscope(pPointer: *mut c_void) -> RingList;
    pub fn ring_vm_api_scopescount(pPointer: *mut c_void) -> c_int;

    pub fn ring_vm_api_cpointercmp(
        pPointer: *mut c_void,
        pList: RingList,
        pList2: RingList,
    ) -> c_int;

    pub fn ring_vm_error(pPointer: *mut c_void, cStr: *const c_char);

    pub fn ring_list_new(nSize: c_uint) -> RingList;
    pub fn ring_list_delete(pList: RingList) -> RingList;
    pub fn ring_list_newlist(pList: RingList) -> RingList;
    pub fn ring_list_getitem(pList: RingList, nIndex: c_uint) -> RingItem;
    pub fn ring_list_gettype(pList: RingList, nIndex: c_uint) -> c_uint;
    pub fn ring_list_getlist(pList: RingList, nIndex: c_uint) -> RingList;

    pub fn ring_list_addint(pList: RingList, nNumber: c_int);
    pub fn ring_list_adddouble(pList: RingList, nNumber: c_double);
    pub fn ring_list_addstring(pList: RingList, cStr: *const c_char);
    pub fn ring_list_addstring2(pList: RingList, cStr: *const c_char, nLen: c_uint);
    pub fn ring_list_addpointer(pList: RingList, pValue: *mut c_void);
    pub fn ring_list_addcpointer(pList: RingList, pGeneral: *mut c_void, cType: *const c_char);

    pub fn ring_list_setint(pList: RingList, nIndex: c_uint, nNumber: c_int);
    pub fn ring_list_setdouble(pList: RingList, nIndex: c_uint, nNumber: c_double);
    pub fn ring_list_setstring(pList: RingList, nIndex: c_uint, cStr: *const c_char);
    pub fn ring_list_setstring2(
        pList: RingList,
        nIndex: c_uint,
        cStr: *const c_char,
        nStrSize: c_uint,
    );
    pub fn ring_list_setpointer(pList: RingList, nIndex: c_uint, pValue: *mut c_void);
    pub fn ring_list_setlist(pList: RingList, nIndex: c_uint);

    pub fn ring_list_insertint(pList: RingList, nPos: c_uint, nNumber: c_int);
    pub fn ring_list_insertdouble(pList: RingList, nPos: c_uint, nNumber: c_double);
    pub fn ring_list_insertstring(pList: RingList, nPos: c_uint, cStr: *const c_char);
    pub fn ring_list_insertstring2(
        pList: RingList,
        nPos: c_uint,
        cStr: *const c_char,
        nStrSize: c_uint,
    );
    pub fn ring_list_insertpointer(pList: RingList, nPos: c_uint, pValue: *mut c_void);
    pub fn ring_list_insertlist(pList: RingList, nPos: c_uint) -> RingList;

    pub fn ring_list_deleteitem(pList: RingList, nIndex: c_uint);
    pub fn ring_list_deleteallitems(pList: RingList);

    pub fn ring_list_isnumber(pList: RingList, nIndex: c_uint) -> c_uint;
    pub fn ring_list_isstring(pList: RingList, nIndex: c_uint) -> c_uint;
    pub fn ring_list_islist(pList: RingList, nIndex: c_uint) -> c_uint;
    pub fn ring_list_ispointer(pList: RingList, nIndex: c_uint) -> c_uint;

    pub fn ring_list_findstring(pList: RingList, cStr: *const c_char, nColumn: c_uint) -> c_uint;
    pub fn ring_list_finddouble(pList: RingList, nNum1: c_double, nColumn: c_uint) -> c_uint;
    pub fn ring_list_findpointer(pList: RingList, pPointer: *mut c_void) -> c_uint;

    pub fn ring_list_copy(pNewList: RingList, pList: RingList);
    pub fn ring_list_swap(pList: RingList, x: c_uint, y: c_uint);
    pub fn ring_list_swaptwolists(pList1: RingList, pList2: RingList);
    pub fn ring_list_genarray(pList: RingList);
    pub fn ring_list_deletearray(pList: RingList);
    pub fn ring_list_genhashtable(pList: RingList);
    pub fn ring_list_genhashtable2(pList: RingList);
    pub fn ring_list_print(pList: RingList);
    pub fn ring_list_print2(pList: RingList, nDecimals: c_uint);

    pub fn ring_list_sortstr(
        pList: RingList,
        left: c_uint,
        right: c_uint,
        nColumn: c_uint,
        cAttribute: *const c_char,
    );
    pub fn ring_list_isobject(pList: RingList) -> c_uint;
    pub fn ring_list_iscpointerlist(pList: RingList) -> c_uint;
    pub fn ring_list_cpointercmp(pList: RingList, pList2: RingList) -> c_uint;
    pub fn ring_list_addringpointer(pList: RingList, pValue: *mut c_void);
    pub fn ring_list_printobj(pList: RingList, nDecimals: c_uint);

    pub fn ring_list_newitem(pList: RingList);
    pub fn ring_list_insertitem(pList: RingList, x: c_uint);

    pub fn ring_list_setfuncpointer(
        pList: RingList,
        nIndex: c_uint,
        pFunc: extern "C" fn(*mut c_void),
    );
    pub fn ring_list_addfuncpointer(pList: RingList, pFunc: extern "C" fn(*mut c_void));
    pub fn ring_list_isfuncpointer(pList: RingList, nIndex: c_uint) -> c_uint;
    pub fn ring_list_insertfuncpointer(
        pList: RingList,
        nPos: c_uint,
        pFunc: extern "C" fn(*mut c_void),
    );

    pub fn ring_string_new(cStr: *const c_char) -> RingString;
    pub fn ring_string_new2(cStr: *const c_char, nStrSize: c_uint) -> RingString;
    pub fn ring_string_delete(pString: RingString) -> RingString;
    pub fn ring_string_set(pString: RingString, cStr: *const c_char);
    pub fn ring_string_set2(pString: RingString, cStr: *const c_char, nStrSize: c_uint);
    pub fn ring_string_add(pString: RingString, cStr: *const c_char);
    pub fn ring_string_add2(pString: RingString, cStr: *const c_char, nStrSize: c_uint);
    pub fn ring_string_setfromint(pString: RingString, x: c_int);
    pub fn ring_string_size(pString: RingString) -> c_uint;
    pub fn ring_string_print(pString: RingString);
    pub fn ring_string_strdup(cStr: *const c_char) -> *mut c_char;

    pub fn ring_state_new() -> RingState;
    pub fn ring_state_init() -> RingState;
    pub fn ring_state_delete(pRingState: RingState) -> RingState;
    pub fn ring_state_runcode(pRingState: RingState, cStr: *const c_char);
    pub fn ring_state_findvar(pRingState: RingState, cStr: *const c_char) -> RingList;
    pub fn ring_state_newvar(pRingState: RingState, cStr: *const c_char) -> RingList;
    pub fn ring_state_runfile(pRingState: RingState, cFileName: *const c_char) -> c_int;
    pub fn ring_state_runstring(pRingState: RingState, cString: *const c_char) -> c_int;

    pub fn ring_state_malloc(pRingState: RingState, nSize: libc::size_t) -> *mut c_void;
    pub fn ring_state_calloc(
        pRingState: RingState,
        nCount: libc::size_t,
        nSize: libc::size_t,
    ) -> *mut c_void;
    pub fn ring_state_realloc(
        pRingState: RingState,
        pPtr: *mut c_void,
        nOldSize: libc::size_t,
        nNewSize: libc::size_t,
    ) -> *mut c_void;
    pub fn ring_state_free(pRingState: RingState, pPtr: *mut c_void);

    pub fn ring_item_getnumber(pItem: RingItem) -> c_double;

    pub fn ring_general_fexists(cFileName: *const c_char) -> c_int;
    pub fn ring_general_currentdir(cDirPath: *mut c_char) -> c_int;
    pub fn ring_general_exefilename(cDirPath: *mut c_char) -> c_int;
    pub fn ring_general_chdir(cDir: *const c_char) -> c_int;
    pub fn ring_general_exefolder(cDirPath: *mut c_char);
    pub fn ring_general_lower(cStr: *mut c_char) -> *mut c_char;
    pub fn ring_general_upper(cStr: *mut c_char) -> *mut c_char;
    pub fn ring_general_lower2(cStr: *mut c_char, nStrSize: c_uint) -> *mut c_char;
    pub fn ring_general_upper2(cStr: *mut c_char, nStrSize: c_uint) -> *mut c_char;
    pub fn ring_general_numtostring(
        nNum1: c_double,
        cStr: *mut c_char,
        nDecimals: c_int,
    ) -> *mut c_char;
    pub fn ring_general_find(cStr1: *mut c_char, cStr2: *mut c_char) -> *mut c_char;
    pub fn ring_general_find2(
        cStr1: *mut c_char,
        nStrSize1: c_uint,
        cStr2: *mut c_char,
        nStrSize2: c_uint,
    ) -> *mut c_char;

    // Item functions
    pub fn ring_item_new(nItemType: c_uint) -> RingItem;
    pub fn ring_item_delete(pItem: RingItem) -> RingItem;
    pub fn ring_item_settype(pItem: RingItem, nItemType: c_uint);
    pub fn ring_item_setstring(pItem: RingItem, cStr: *const c_char);
    pub fn ring_item_setstring2(pItem: RingItem, cStr: *const c_char, nStrSize: c_uint);
    pub fn ring_item_setdouble(pItem: RingItem, x: c_double);
    pub fn ring_item_setint(pItem: RingItem, x: c_int);
    pub fn ring_item_setpointer(pItem: RingItem, pValue: *mut c_void);
    pub fn ring_item_print(pItem: RingItem);
    pub fn ring_item_init(pItem: RingItem);
    pub fn ring_item_deletecontent(pItem: RingItem);

    // Item array functions
    pub fn ring_itemarray_setint(aItems: *mut Item, nIndex: c_uint, nNumber: c_int);
    pub fn ring_itemarray_setdouble(aItems: *mut Item, nIndex: c_uint, nNumber: c_double);
    pub fn ring_itemarray_setpointer(aItems: *mut Item, nIndex: c_uint, pValue: *mut c_void);
    pub fn ring_itemarray_setstring(aItems: *mut Item, nIndex: c_uint, cStr: *const c_char);
    pub fn ring_itemarray_setstring2(
        aItems: *mut Item,
        nIndex: c_uint,
        cStr: *const c_char,
        nStrSize: c_uint,
    );

    // State utility functions
    pub fn ring_state_log(pRingState: RingState, cStr: *const c_char);
    pub fn ring_state_exit(pRingState: RingState, nExitCode: c_int);
    pub fn ring_state_runobjectfile(pRingState: RingState, cFileName: *mut c_char);
    pub fn ring_state_runobjectstring(
        pRingState: RingState,
        cString: *mut c_char,
        nSize: c_uint,
        cFileName: *const c_char,
    );
    pub fn ring_state_runprogram(pRingState: RingState);
    pub fn ring_state_newbytecode(pRingState: RingState, nSize: c_uint, lLiteral: c_uint);
    pub fn ring_state_runbytecode(pRingState: RingState);

    // VM callback function
    pub fn ring_vm_callfunction(pVM: RingVM, cFuncName: *const c_char);

    // Threading/Mutex functions
    pub fn ring_vm_mutexlock(pVM: RingVM);
    pub fn ring_vm_mutexunlock(pVM: RingVM);
    pub fn ring_vm_mutexdestroy(pVM: RingVM);
    pub fn ring_vm_runcodefromthread(pVM: RingVM, cCode: *const c_char);

    // Advanced VM functions for callbacks
    pub fn ring_vm_loadfunc2(pVM: RingVM, cStr: *const c_char, nPerformance: c_int) -> c_int;
    pub fn ring_vm_call2(pVM: RingVM);
    pub fn ring_vm_fetch(pVM: RingVM);
    pub fn ring_vm_fetch2(pVM: RingVM);

    // VM execution and utilities
    pub fn ring_vm_runcode(pVM: RingVM, cStr: *const c_char);
    pub fn ring_vm_numtostring(pVM: RingVM, nNum: c_double, cStr: *mut c_char) -> *mut c_char;
    pub fn ring_vm_stringtonum(pVM: RingVM, cStr: *const c_char) -> c_double;
    pub fn ring_vm_aftercfunction(pVM: RingVM, pFuncCall: *mut c_void);
    pub fn ring_vm_callfuncwithouteval(pVM: RingVM, cStr: *const c_char, lMethod: c_uint);
    pub fn ring_vm_showbytecode(pVM: RingVM);
    pub fn ring_vm_showerrormessage(pVM: RingVM, cStr: *const c_char);
    pub fn ring_vm_shutdown(pVM: RingVM, nExitCode: c_int);

    // VM threading
    pub fn ring_vm_createthreadstate(pVM: RingVM) -> RingState;
    pub fn ring_vm_deletethreadstate(pVM: RingVM, pState: RingState);
    pub fn ring_vm_bytecodefornewthread(pVM: RingVM, pOldVM: RingVM);

    // Custom mutex functions (all take VM* as first param)
    pub fn ring_vm_custmutexcreate(pVM: RingVM) -> *mut c_void;
    pub fn ring_vm_custmutexdestroy(pVM: RingVM, pMutex: *mut c_void);
    pub fn ring_vm_custmutexlock(pVM: RingVM, pMutex: *mut c_void);
    pub fn ring_vm_custmutexunlock(pVM: RingVM, pMutex: *mut c_void);
    pub fn ring_vm_statecustmutexlock(pState: *mut c_void, nMutex: c_uint);
    pub fn ring_vm_statecustmutexunlock(pState: *mut c_void, nMutex: c_uint);
    pub fn ring_vm_mutexfunctions(
        pVM: RingVM,
        pCreate: Option<extern "C" fn() -> *mut c_void>,
        pLock: Option<extern "C" fn(*mut c_void)>,
        pUnlock: Option<extern "C" fn(*mut c_void)>,
        pDestroy: Option<extern "C" fn(*mut c_void)>,
    );

    // State GC block registration
    pub fn ring_state_registerblock(pRingState: RingState, pStart: *mut c_void, pEnd: *mut c_void);
    pub fn ring_state_unregisterblock(pRingState: RingState, pStart: *mut c_void);
    pub fn ring_state_willunregisterblock(pRingState: RingState, pStart: *mut c_void);

    // State utilities
    pub fn ring_state_cgiheader(pRingState: RingState);

    // VM loading functions
    pub fn ring_vm_loadcfunctions(pRingState: RingState);
    pub fn ring_vm_generallib_loadfunctions(pRingState: RingState);
    pub fn ring_vm_loadcode(pVM: RingVM);
}
