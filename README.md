# diff
diff two file, show in terminal, save as txt or html

### Args
```rust
Usage: dif2 [-a <old>] [-b <new>] [-r <res>] [-c <color>] [-w <width>] [-t <batch>] [-d] [-n <outname>] [-o <outpath>]

highlight diff

Options:
  -a, --old         old file
  -b, --new         new file
  -r, --res         result type, support: terminal(default), txt, html
  -c, --color       highlight color for left, both, right, support: Black, Blue,
                    Green, Red, Cyan, Magenta, Yellow, White, rgb,x,x,x,
                    default: Red:White:Green
  -w, --width       line number width, default: 5
  -t, --batch       batch number, default: 0 (all result)
  -d, --diff        only show diff result
  -n, --outname     outname, default: diff_result
  -o, --outpath     outpath, default: ./
  --help            display usage information
```
### Example
1. print in terminal
   ```shell
   dif -a test1.rs -b test2.rs
   ```
2. save as txt
   ```shell
   dif -a test1.rs -b test2.rs -r txt
3. save as html
   ```shell
   dif -a test1.rs -b test2.rs -r html
   ```
