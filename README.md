# ZIP rebuild

`zip-rebuild` is a library written in Rust that allows you to take apart a ZIP file and then rebuild it bit-perfect. It
uses the C++ [preflate](https://github.com/deus-libri/preflate) library to rebuild Deflate streams, while all other
streams are kept as-is (e.g. it's useless to run this on a ZSTD or ZipCrypto zip file).

Example usage:

Let's say you have a file called `some-file.zip` with `a.txt` stored and `b.txt` deflated inside.

```console
$ zip-rebuild-simple dump some-file.zip
```

This will result in the following files:

```
some-file.zip <- This file has existed previously, all others have just been created

some-file.zip.rebuild_info.json
some-file/some-file.zip.headers
some-file/a.txt
some-file/b.txt <- Decompressed copy of b.txt
some-file/b.txt.preflate
```

Now you can compress those files with a stronger algorithm while still being able to rebuild the original zip. See down
below for a test on `silesia.zip`

Once you have the files above, you can rebuild the original zip file by running:

```console
$ zip-rebuild-simple rebuild some-file.zip.rebuild_info.json --output some-file.rebuilt.zip
```

The recreated file `some-file.rebuilt.zip` is identical to the original `some-file.zip` we provided.

If the `--output` argument is omitted, the original file name will be restored. It is kept in the rebuild info json file
as one of the fields.

## Example on silesia.zip

The zip file of [Silesia compression corpus](https://sun.aei.polsl.pl//~sdeor/index.php?page=silesia) is 68182744 bytes.

Let's try the naive approach first. After running `zstd --ultra -22 --long=31 silesia.zip` the size of the resulting
`silesia.zip.zst` is 67953257 bytes. 

That's a 229487 byte difference or, in other words, the zstd-compressed zip is 99.66% the size of the original. 

If we dump the zip with `zip-rebuild-simple dump silesia.zip` we get 212212858 bytes of data - all decompressed files
inside the zip with their rebuild info. Let's put that into a `tar.zst`, with the same zstd arguments as above.

The resulting `silesia.tar.zst` containing decompressed files and rebuild info is 52724874 bytes, or 77.33% or the
original `silesia.zip`

The drawback of this is that `zip-rebuild-simple` also has a size of its own and rebuilding the original zip can take
a while. 

## Usage of zip-rebuild-advanced to dump/rebuild multiple files

Let's say you have 2 files with similar contents, `file1.zip` and `file2.zip`. 

To dump them both into one folder, deduplicating the identical files, use `zip-rebuild-advanced`, like so:

```
original/file1.zip
original/file2.zip
data/
rebuild_info/
```

To dump the files:

```console
$ zip-rebuild-advanced dump-multiple "original/*.zip" --output-dir data --rebuild-info rebuild_info
```

You will have the dumped data in `data/`, with the filename of each file being its BLAKE3 hash (no extension) and the
rebuild JSONs in `rebuild_info/`.

To rebuild the files:

```console
$ zip-rebuild-advanced rebuild-multiple --input-dir data --output-dir rebuilt_zips rebuild_info
```

This will result in a directory `rebuilt_zips` being created with rebuilt `file1.zip` and `file2.zip` inside.

### Caching

Because both zips can keep identical files, it makes sense to recompress them only once and cache the recompression
results for better rebuilding speeds. To do that add `--cache-dir cache` to the rebuild command. `cache/` will be
created, used as a cache then deleted. To keep it add `--keep-cache`.