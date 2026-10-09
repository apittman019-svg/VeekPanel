"""Minimal tray-host fixture on the harness's disposable D-Bus only."""
import os
from pathlib import Path
import gi
gi.require_version('Gio','2.0')
from gi.repository import Gio, GLib

if not Path(os.environ.get('XDG_RUNTIME_DIR','')).name.startswith('veek-native-gui-'):
    raise SystemExit('Requires the private native GUI harness')
if os.environ.get('VEEK_PRIVATE_DBUS_TEST')!='1':
    raise SystemExit('Requires explicit disposable session-bus opt-in')
bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
result=bus.call_sync('org.freedesktop.DBus','/org/freedesktop/DBus','org.freedesktop.DBus','RequestName',
    GLib.Variant('(su)',('org.kde.StatusNotifierWatcher',4)),GLib.VariantType('(u)'),0,2000,None)
if result.unpack()[0]!=1:raise SystemExit('Tray name already owned; refusing to replace it')
xml='''<node><interface name="org.kde.StatusNotifierWatcher">
<method name="RegisterStatusNotifierItem"><arg type="s" direction="in"/></method>
<method name="RegisterStatusNotifierHost"><arg type="s" direction="in"/></method>
<property name="IsStatusNotifierHostRegistered" type="b" access="read"/>
<property name="RegisteredStatusNotifierItems" type="as" access="read"/>
<property name="ProtocolVersion" type="i" access="read"/>
</interface></node>'''
def method(connection,sender,path,interface,name,parameters,invocation):
    invocation.return_value(GLib.Variant('()',()))
def prop(connection,sender,path,interface,name):
    return {'IsStatusNotifierHostRegistered':GLib.Variant('b',True),
            'RegisteredStatusNotifierItems':GLib.Variant('as',[]),
            'ProtocolVersion':GLib.Variant('i',0)}[name]
bus.register_object('/StatusNotifierWatcher',Gio.DBusNodeInfo.new_for_xml(xml).interfaces[0],method,prop,None)
print('private tray fixture ready',flush=True)
GLib.MainLoop().run()
