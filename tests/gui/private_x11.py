"""Native input/resize fallback for the harness-owned Xvfb display only."""
import ctypes as c


def perform(display, operation, *args):
    number = int(display.removeprefix(':'))
    if not 110 <= number < 180:
        raise RuntimeError('Refusing to operate outside the private test display range')
    x = c.CDLL('libX11.so.6')
    signatures = {
        'XOpenDisplay': ([c.c_char_p], c.c_void_p),
        'XDefaultRootWindow': ([c.c_void_p], c.c_ulong),
        'XQueryTree': ([c.c_void_p,c.c_ulong,c.POINTER(c.c_ulong),c.POINTER(c.c_ulong),c.POINTER(c.POINTER(c.c_ulong)),c.POINTER(c.c_uint)],c.c_int),
        'XFetchName': ([c.c_void_p,c.c_ulong,c.POINTER(c.c_char_p)],c.c_int),
        'XFree': ([c.c_void_p],c.c_int),
        'XSetInputFocus': ([c.c_void_p,c.c_ulong,c.c_int,c.c_ulong],c.c_int),
        'XWarpPointer': ([c.c_void_p,c.c_ulong,c.c_ulong,c.c_int,c.c_int,c.c_uint,c.c_uint,c.c_int,c.c_int],c.c_int),
        'XResizeWindow': ([c.c_void_p,c.c_ulong,c.c_uint,c.c_uint],c.c_int),
        'XStringToKeysym': ([c.c_char_p],c.c_ulong),
        'XKeysymToKeycode': ([c.c_void_p,c.c_ulong],c.c_ubyte),
        'XInternAtom': ([c.c_void_p,c.c_char_p,c.c_int],c.c_ulong),
        'XSendEvent': ([c.c_void_p,c.c_ulong,c.c_int,c.c_long,c.c_void_p],c.c_int),
        'XSync': ([c.c_void_p,c.c_int],c.c_int),
        'XCloseDisplay': ([c.c_void_p],c.c_int),
    }
    for name,(params,result) in signatures.items():
        fn=getattr(x,name);fn.argtypes=params;fn.restype=result
    d=x.XOpenDisplay(display.encode())
    if not d: raise RuntimeError('Private display unavailable')
    try:
        root=c.c_ulong();parent=c.c_ulong();children=c.POINTER(c.c_ulong)();count=c.c_uint()
        if not x.XQueryTree(d,x.XDefaultRootWindow(d),c.byref(root),c.byref(parent),c.byref(children),c.byref(count)):
            raise RuntimeError('Cannot query private windows')
        found=[]
        try:
            for window in children[:count.value]:
                name=c.c_char_p()
                if x.XFetchName(d,window,c.byref(name)) and name:
                    try:
                        if name.value==b'VeekPanel':found.append(window)
                    finally:x.XFree(c.cast(name,c.c_void_p))
        finally:
            if children:x.XFree(children)
        if len(found)!=1:raise RuntimeError(f'Expected one private VeekPanel window, found {len(found)}')
        window=found[0]
        if operation=='resize':x.XResizeWindow(d,window,*args)
        elif operation=='close':
            class Data(c.Union):
                _fields_=[('l',c.c_long*5),('b',c.c_char*20)]
            class ClientMessage(c.Structure):
                _fields_=[('type',c.c_int),('serial',c.c_ulong),('send_event',c.c_int),
                          ('display',c.c_void_p),('window',c.c_ulong),('message_type',c.c_ulong),
                          ('format',c.c_int),('data',Data)]
            class Event(c.Union):
                _fields_=[('client',ClientMessage),('padding',c.c_long*24)]
            event=Event();event.client.type=33;event.client.window=window
            event.client.message_type=x.XInternAtom(d,b'WM_PROTOCOLS',0);event.client.format=32
            event.client.data.l[0]=x.XInternAtom(d,b'WM_DELETE_WINDOW',0)
            if not x.XSendEvent(d,window,0,0,c.byref(event)):raise RuntimeError('Native close was not sent')
        else:
            x.XSetInputFocus(d,window,1,0)
            if operation in ('key','key_down','key_up','click'):
                xt=c.CDLL('libXtst.so.6')
                xt.XTestFakeKeyEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong]
                if operation=='click':
                    xt.XTestFakeButtonEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong]
                    x.XWarpPointer(d,0,window,0,0,0,0,int(args[0]),int(args[1]))
                    xt.XTestFakeButtonEvent(d,1,1,0);xt.XTestFakeButtonEvent(d,1,0,0)
                else:
                    code=x.XKeysymToKeycode(d,x.XStringToKeysym(args[0].encode()))
                    if not code:raise RuntimeError('Unknown test key')
                    if operation!='key_up':xt.XTestFakeKeyEvent(d,code,1,0)
                    if operation!='key_down':xt.XTestFakeKeyEvent(d,code,0,0)
            elif operation!='focus':raise RuntimeError('Unknown test operation')
        x.XSync(d,0)
    finally:x.XCloseDisplay(d)
