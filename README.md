# osv2osv
Converting OSV csv back to (byte identical) [EML_NL](https://www.kiesraad.nl/verkiezingen/osv-en-eml/eml-standaard)

Since some people have scripts for EML, but municipalities are only adviced to publish OSV export in CSV.
This converts OSV csv back to EML.XML (with a fixed version `2.23.6` comment and election date + `T12:34:56.789` for `kr:CreationDateTime`).

## Usage
```bash
cat testdata/osv4-3_Telling_PS2019_Fryslan_gemeente_Tytsjerksteradiel.csv | cargo run > generated.eml.xml
# since we cannot generate the CreationDateTime from the csv,
# manually rewrite the placeholder to the correct time before checking the sha256hash
sed 's/20T12:34:56.789/21T06:57:22.834/' generated.eml.xml | sha256sum
5b095dae163a90b31a7415bea81b231823e01ae1024aa8c82f03dbd256b78369
```

## Note
Would have loved to have used `serde-xml-rs` but since there is no attribute serialization ([issue #49](https://github.com/RReverser/serde-xml-rs/issues/49)) it's not yet usable for this project.

## License

[CC-0](https://creativecommons.org/publicdomain/zero/1.0/)

## Working on
- brute force search the `kr:CreationDateTime` and IVU program/version based on a SHA256 hash input (from the N11 footer)

## Future
- convert EML to OSV csv (will need candidates eml for this)
- support other elections that AB/PS/EP 2019