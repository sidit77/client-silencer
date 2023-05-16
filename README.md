# client-silencer
This is an experiment in function hooking using [IAT Hooking](https://relearex.wordpress.com/2017/12/26/hooking-series-part-i-import-address-table-hooking/) and Rust.

More specifically this program overwrites the [SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos) function to prevent the injected program from setting one of its windows to always-on-top.

## Running

```bash
git clone https://github.com/sidit77/client-silencer.git
cd client-silencer
cargo run --release
```

## License
MIT License