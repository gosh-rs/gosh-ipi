// [[file:../ipi.note::d2086cfc][d2086cfc]]
use super::*;

use bytes::{Buf, BufMut};
use bytes::{Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

type EncodedResult = Result<(), std::io::Error>;

const HEADER_SIZE: usize = 12;

use gchemol::units::{Bohr, Hartree};
// d2086cfc ends here
