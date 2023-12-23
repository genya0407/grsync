# grsync

Unofficial image downloader for [RICOH GR-III](https://www.ricoh-imaging.co.jp/english/products/gr-3/).

This tool is heavily inspired by https://github.com/clyang/GRsync

## Usage

1. Enable GR-III's Wi-Fi access point, and connect to it
2. Run command (see below)

## Command examples

```bash
# Downloads images into ./downloaded_photos directory.
# Already downloaded ones are skipped.
$ grsync

# Downloads images into ./hoge directory
$ grsync -o hoge

# Downloads all images, including already downloaded ones.
$ grsync --force
```
