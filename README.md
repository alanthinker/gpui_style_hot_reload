 

 

# Quick Start
This project is a demonstration.  
After you run the program, modifying the style.pjson file and saving, it will immediately apply the latest styles to the UI.
But I've only implemented the most commonly used styles. If you'd like to support additional styles, you can simply modify the source code yourself. it's very straightforward.

# About hot-reload layout
I also developed a feature for hot-reloading layouts, but considering that controlling layouts with JSON is less flexible and more restrictive compared to using Rust code, I eventually abandoned it.  
Hot-reloading only the styles is sufficient for meeting the requirements of rapid UI development in most cases.