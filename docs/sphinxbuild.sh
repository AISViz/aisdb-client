SCRIPTPATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
RSTSOURCEDIR="${SCRIPTPATH}/source"
HTMLOUTPUTDIR="${SCRIPTPATH}/html"
PKGDIR="${SCRIPTPATH}/../aisdb"
ROOTDIR="${SCRIPTPATH}/.."


rm -rf "$HTMLOUTPUTDIR"
mkdir -p "${RSTSOURCEDIR}/api"
mkdir -p "${HTMLOUTPUTDIR}/_images"
[[ ! -z `ls -A "${RSTSOURCEDIR}/api"` ]] && rm ${RSTSOURCEDIR}/api/*
cp "$ROOTDIR/readme.rst" "${RSTSOURCEDIR}/readme.rst"
sphinx-apidoc --separate --force --implicit-namespaces --module-first --no-toc -q -o "${RSTSOURCEDIR}/api" "${PKGDIR}"
python -m sphinx -a -j auto -q -b=html "${RSTSOURCEDIR}" "${HTMLOUTPUTDIR}"
cp "${RSTSOURCEDIR}/db_schema.png" "$HTMLOUTPUTDIR/_images/"
cp "${RSTSOURCEDIR}/scriptoutput.png" "$HTMLOUTPUTDIR/_images/"
cargo doc \
  --document-private-items \
  --frozen \
  --manifest-path="$ROOTDIR/aisdb_rust/Cargo.toml" \
  --no-deps \
  --offline \
  --package=aisdb \
  --release \
  --target-dir="$HTMLOUTPUTDIR/rust" 
  #--profile=test \
