## WCA developer to tsv
This is a tool for converting WCA developer export sql dumps to tsvs. In theory it might also work for other MySql dumps. It works for the current version as of the day before this release. There are absolutely no guarantees about any previous or future versions nor the correctness of the output.

## Usage
`wca-developer-to-tsv <path-to-sql>`

The output is written in the ˝tables˝ folder. If this already exists it will be overwritten with no warning.

## Install
`cargo install --git https://github.com/Daniel-Anker-Hermansen/wca-developer-to-tsv`
