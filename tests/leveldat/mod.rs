use bnbt::codec::{Endian::Little, NBTCodec, NBTCodecTrait};
use std::{env::current_dir, fs, io::BufReader};

#[test]
fn simple_leveldat_test() {
    let nbt_reader = NBTCodec { endian: Little };

    let cur_dir = current_dir().unwrap();

    let result = fs::read(
        cur_dir
            .join("tests")
            .join("resources")
            .join("level.dat")
            .as_path(),
    )
    .unwrap();

    let mut buf_reader = BufReader::new(result.as_slice());

    let _empty_compound_tag = nbt_reader.read_tag(&mut buf_reader);
    let _size = nbt_reader.read_i32(&mut buf_reader);
    let tag = nbt_reader.read_tag(&mut buf_reader);

    println!("{:?}", tag);
}
