void unknown(int param_1)
{
  if (SysCall(0,0x16,param_1) == 3) {
    SysCall(0,0x13,param_1);
  }
  return;
}
int get_module(string name) {
    int handle = SysCall(0x0, 0x10, name);
    if (!handle) {
        SysCall(0x0, 0x3, "slFindModule: module not found", name);
        SysCall(0x0, 0x1, "ERROR: slFindModule\n");
    }
    return handle;
}

static void ACTIVATE(int param_1)

{
  return;
}


static void DEACTIVATE(int param_1)

{
  return;
}

static void CLOSE(int param_1)

{
  int object_manager;
  int global_manager;
  int gf0002_flag;
  int gate_id;
  int gate_object;

  object_manager = get_module("ObjectManager");
  gate_id = SysCall(0,0x15,0,param_1);
  gate_object = SysCall(0,0x15,1,object_manager,gate_id);
  global_manager = get_module("GlobalManager");
  gf0002_flag = SysCall(0,0x15,1,global_manager,"GF0002");
  if (gf0002_flag == 1) {
    SysCall(0,0x15,0,gate_object,2);
  }
  else {
    SysCall(0,0x15,4,param_1);
    SysCall(0,0x15,1,gate_object,0);
  }
  return;
}

static void RUN(int param_1)

{
  int object_manager;
  int gate_id;
  int gate_object;
  int uVar3;

  object_manager = get_module("ObjectManager");
  gate_id = SysCall(0,0x15,0,param_1);
  gate_object = SysCall(0,0x15,1,object_manager,gate_id);
  uVar3 = SysCall(0,0x15,4,param_1);
  SysCall(0,0x15,0,uVar3,param_1,1);
  SysCall(0,0x15,0x28,gate_object,0x4fb,0,0x3f800000,0x3f800000);
  while (true) {
   int iVar1 = SysCall(0,0x15,4,uVar3,param_1);
   if(iVar1 == 0){
    break;
   }
   Pause(1);
   }
  SysCall(0,0x15,0,uVar3,param_1,2);
  SysCall(0,0x15,0,gate_object,2);
  return;
}


static void OPEN(int param_1)

{
  int uVar1;

  uVar1 = SysCall(0,0x15,4,param_1);
  SysCall(0,0x15,0,uVar1,param_1,2);
  uVar1 = SysCall(0,0x15,0x1e,param_1);
  SysCall(0,0x15,4,uVar1,1,0);
  SysCall(0,0x15,4,uVar1,1,1);
  return;
}