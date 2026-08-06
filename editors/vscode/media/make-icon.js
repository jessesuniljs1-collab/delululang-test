// Generate the extension icon as a PNG, with no image library.
//
// Written rather than drawn so the icon is reproducible from source: a binary blob committed with
// no way to regenerate it is a small piece of the repository nobody can change.
//
// The mark: a bracket pair enclosing a single dot. DeluluLang's whole idea is that a program holds
// nothing except what it is handed, so the icon is a boundary with something contained inside it.

const zlib = require("zlib");
const fs = require("fs");

const S = 128;
const px = Buffer.alloc(S * S * 4);

// Palette: a deep indigo field, a bright mint boundary, a warm dot.
const BG = [24, 24, 37, 255];
const FG = [148, 226, 213, 255];
const DOT = [249, 226, 175, 255];

function set(x, y, c) {
  if (x < 0 || y < 0 || x >= S || y >= S) return;
  const i = (y * S + x) * 4;
  px[i] = c[0];
  px[i + 1] = c[1];
  px[i + 2] = c[2];
  px[i + 3] = c[3];
}
function rect(x0, y0, w, h, c) {
  for (let y = y0; y < y0 + h; y++) for (let x = x0; x < x0 + w; x++) set(x, y, c);
}
function disc(cx, cy, r, c) {
  for (let y = cy - r; y <= cy + r; y++)
    for (let x = cx - r; x <= cx + r; x++)
      if ((x - cx) ** 2 + (y - cy) ** 2 <= r * r) set(x, y, c);
}

// Field, with rounded corners so it sits well among other extension icons.
rect(0, 0, S, S, BG);
const R = 22;
for (const [cx, cy] of [[R, R], [S - R - 1, R], [R, S - R - 1], [S - R - 1, S - R - 1]]) {
  for (let y = 0; y < S; y++)
    for (let x = 0; x < S; x++) {
      const nearCorner =
        (x < R && y < R && cx === R && cy === R) ||
        (x > S - R && y < R && cx === S - R - 1 && cy === R) ||
        (x < R && y > S - R && cx === R && cy === S - R - 1) ||
        (x > S - R && y > S - R && cx === S - R - 1 && cy === S - R - 1);
      if (nearCorner && (x - cx) ** 2 + (y - cy) ** 2 > R * R) set(x, y, [0, 0, 0, 0]);
    }
}

// The boundary: two brackets, drawn as three strokes each.
const T = 9; // stroke thickness
const top = 30;
const bot = S - 30;
const height = bot - top;

// Left bracket  [
rect(30, top, T, height, FG);
rect(30, top, 24, T, FG);
rect(30, bot - T, 24, T, FG);
// Right bracket ]
rect(S - 30 - T, top, T, height, FG);
rect(S - 30 - 24, top, 24, T, FG);
rect(S - 30 - 24, bot - T, 24, T, FG);

// What is held.
disc(S / 2, S / 2, 11, DOT);

// --- PNG encoding ---------------------------------------------------------------------------
const CRC = (() => {
  const t = new Int32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c;
  }
  return (buf) => {
    let c = 0xffffffff;
    for (const b of buf) c = t[(c ^ b) & 0xff] ^ (c >>> 8);
    return (c ^ 0xffffffff) >>> 0;
  };
})();

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const td = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(CRC(td));
  return Buffer.concat([len, td, crc]);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(S, 0);
ihdr.writeUInt32BE(S, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // RGBA
// 10,11,12 = compression, filter, interlace = 0

// One filter byte (0 = none) per scanline.
const raw = Buffer.alloc(S * (S * 4 + 1));
for (let y = 0; y < S; y++) {
  raw[y * (S * 4 + 1)] = 0;
  px.copy(raw, y * (S * 4 + 1) + 1, y * S * 4, (y + 1) * S * 4);
}

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr),
  chunk("IDAT", zlib.deflateSync(raw, { level: 9 })),
  chunk("IEND", Buffer.alloc(0)),
]);

fs.writeFileSync(process.argv[2], png);
console.log(`wrote ${process.argv[2]} — ${png.length} bytes, ${S}x${S} RGBA`);
