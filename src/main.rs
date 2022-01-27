use std::{fs::File, io::BufReader};

use helper::Helper;
use ntfs::Ntfs;
use sector_reader::SectorReader;

mod helper;
mod sector_reader;

fn main() {
    let f = File::open(r"\\.\C:").unwrap();
    let sr = SectorReader::new(f, 512).unwrap();
    let mut fs = BufReader::new(sr);
    let mut ntfs = Ntfs::new(&mut fs).unwrap();
    ntfs.read_upcase_table(&mut fs).unwrap();

    let root = ntfs.root_directory(&mut fs).unwrap();

    let mut helper = Helper::new(fs, &ntfs, root);

    helper.cd("Tools");
    helper.cd("FanCtrl");
    helper.ls();
}
