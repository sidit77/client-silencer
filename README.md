# client-silencer
This is an experiment in function hooking using [IAT Hooking](https://relearex.wordpress.com/2017/12/26/hooking-series-part-i-import-address-table-hooking/) and Rust.

More specifically this program overwrites the [SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos) function to prevent the injected program from setting one of its windows to always-on-top.

## Building

```bash
git clone https://github.com/sidit77/client-silencer.git
cd client-silencer
cargo build --release
```

## Running
After building the `target/release/` folder should contain the two files `dll-injector.exe` and `client_hook.dll`. 

Double-click `dll-injector.exe` while the target program is open to inject `client_hook.dll` into it. 

All changes will disappear once the program is closed.

## License
MIT License