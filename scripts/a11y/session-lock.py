#!/usr/bin/python3
"""Report whether the GUI session can host an automated screen-reader run.

Exit 0 when the session is logged in and unlocked, 1 otherwise (and print
the reason). Keys come from CGSessionCopyCurrentDictionary, the same source
`/System/Library/CoreGraphics` hands every app: usable without any TCC
grant, which matters because the whole point of the caller is that it has
none.

`CGSSessionScreenIsLocked` is absent while the screen is unlocked (it is
only written when a lock is in force), so absence means "go".
"""

import ctypes
import sys

core_graphics = ctypes.CDLL(
    "/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics"
)
core_foundation = ctypes.CDLL(
    "/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation"
)

core_graphics.CGSessionCopyCurrentDictionary.restype = ctypes.c_void_p

core_foundation.CFCopyDescription.restype = ctypes.c_void_p
core_foundation.CFCopyDescription.argtypes = [ctypes.c_void_p]
core_foundation.CFStringGetCString.restype = ctypes.c_bool
core_foundation.CFStringGetCString.argtypes = [
    ctypes.c_void_p,
    ctypes.c_char_p,
    ctypes.c_long,
    ctypes.c_uint32,
]
core_foundation.CFRelease.argtypes = [ctypes.c_void_p]

UTF8 = 0x08000100

session = core_graphics.CGSessionCopyCurrentDictionary()
if not session:
    print("no GUI session (the Mac is at the login window or headless)")
    sys.exit(1)

description = core_foundation.CFCopyDescription(session)
if not description:
    print("could not read the session dictionary")
    sys.exit(1)

buffer = ctypes.create_string_buffer(8192)
if not core_foundation.CFStringGetCString(description, buffer, 8192, UTF8):
    print("could not decode the session dictionary")
    sys.exit(1)
core_foundation.CFRelease(description)

text = buffer.value.decode("utf-8", "replace")

if "kCGSessionLoginDoneKey = 0" in text:
    print("no logged-in user session yet")
    sys.exit(1)
if "CGSSessionScreenIsLocked = 1" in text:
    print("the screen is locked")
    sys.exit(1)

print("unlocked")
sys.exit(0)
