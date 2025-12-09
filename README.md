
[中文翻译](https://github.com/alanthinker/gpui_style_hot_reload/blob/main/README-cn.md)

# About Hot Reloading Styles
This project is a demonstration.  
After running the program (`cargo run`), modify and save the `styles.pjson` file in the root directory. The updated styles will immediately be applied to the UI.  
However, I've only implemented the most commonly used styles. If you want to support additional styles, you can directly modify the source code—it's very straightforward.

# About Hot Reloading Layouts
I've also developed a hot-reload layout feature. However, considering that controlling layouts via JSON is less flexible and more restrictive compared to using Rust code, this approach is better-suited for specific scenarios only.

Main features implemented:
* Hot reloading of layouts
* Data binding to top-level fields of an entity
* Connecting backend Rust events
* Embedding UI components written in Rust code into the pjson layout files

How to run:  
(You must switch to the `examples/layout_demo` directory to run the example, as the file paths used are relative.)
```
cd examples/layout_demo
cargo run
```
Then modify either `layout.pjson` or `styles.pjson`, save the file, and observe the changes.
