use criterion::{criterion_group, criterion_main, Criterion};
const RAW_DATA: &[u8; 901823] = include_bytes!("automerge-paper.json.gz");

#[cfg(feature = "fuzzing")]
mod run {
    use std::io::Read;

    use super::*;
    use arbitrary::Unstructured;
    use flate2::read::GzDecoder;
    use loro_core::fuzz::test_multi_sites;
    use loro_core::fuzz::Action;
    use loro_core::LoroCore;
    use rand::Rng;
    use rand::SeedableRng;
    use serde_json::Value;

    pub fn two_client_edits(c: &mut Criterion) {
        let mut rgn = rand::rngs::StdRng::seed_from_u64(0);
        let mut bytes = Vec::new();
        for _ in 0..1000 {
            bytes.push(rgn.gen::<u8>());
        }

        let mut gen = Unstructured::new(&bytes);
        let actions = gen.arbitrary::<[Action; 200]>().unwrap();
        c.bench_function("random text edit 2 sites", |b| {
            b.iter(|| test_multi_sites(2, actions.clone().into()))
        });

        c.bench_function("random text edit 8 sites", |b| {
            b.iter(|| test_multi_sites(8, actions.clone().into()))
        });
    }

    pub fn b4(c: &mut Criterion) {
        let mut d = GzDecoder::new(&RAW_DATA[..]);
        let mut s = String::new();
        d.read_to_string(&mut s).unwrap();
        let json: Value = serde_json::from_str(&s).unwrap();
        let txns = json.as_object().unwrap().get("txns");
        println!("{}", txns.unwrap().as_array().unwrap().len());
        c.bench_function("B4", |b| {
            b.iter(|| {
                let mut loro = LoroCore::default();
                let mut text = loro.get_or_create_root_text("text").unwrap();
                for txn in txns.unwrap().as_array().unwrap() {
                    let patches = txn
                        .as_object()
                        .unwrap()
                        .get("patches")
                        .unwrap()
                        .as_array()
                        .unwrap();
                    for patch in patches {
                        let pos = patch[0].as_u64().unwrap() as usize;
                        let del_here = patch[1].as_u64().unwrap() as usize;
                        let ins_content = patch[2].as_str().unwrap();
                        text.delete(pos, del_here);
                        text.insert(pos, ins_content);
                    }
                }
            })
        });

        let mut b = c.benchmark_group("sync");
        b.sample_size(10);
        b.bench_function("B4Sync", |b| {
            b.iter(|| {
                let mut loro = LoroCore::default();
                let mut loro_b = LoroCore::default();
                for txn in txns.unwrap().as_array().unwrap() {
                    let mut text = loro.get_or_create_root_text("text").unwrap();
                    let patches = txn
                        .as_object()
                        .unwrap()
                        .get("patches")
                        .unwrap()
                        .as_array()
                        .unwrap();
                    for patch in patches {
                        let pos = patch[0].as_u64().unwrap() as usize;
                        let del_here = patch[1].as_u64().unwrap() as usize;
                        let ins_content = patch[2].as_str().unwrap();
                        text.delete(pos, del_here);
                        text.insert(pos, ins_content);
                    }

                    drop(text);
                    loro_b.import(loro.export(loro_b.vv()));
                }
            })
        });

        b.bench_function("B4Parallel", |b| {
            b.iter(|| {
                let mut loro = LoroCore::default();
                let mut loro_b = LoroCore::default();
                let mut i = 0;
                for txn in txns.unwrap().as_array().unwrap() {
                    i += 1;
                    if i > 1000 {
                        break;
                    }

                    let mut text = loro.get_or_create_root_text("text").unwrap();
                    let patches = txn
                        .as_object()
                        .unwrap()
                        .get("patches")
                        .unwrap()
                        .as_array()
                        .unwrap();
                    for patch in patches {
                        let pos = patch[0].as_u64().unwrap() as usize;
                        let del_here = patch[1].as_u64().unwrap() as usize;
                        let ins_content = patch[2].as_str().unwrap();
                        text.delete(pos, del_here);
                        text.insert(pos, ins_content);
                    }

                    drop(text);
                    let mut text = loro_b.get_or_create_root_text("text").unwrap();
                    for patch in patches {
                        let pos = patch[0].as_u64().unwrap() as usize;
                        let del_here = patch[1].as_u64().unwrap() as usize;
                        let ins_content = patch[2].as_str().unwrap();
                        text.delete(pos, del_here);
                        text.insert(pos, ins_content);
                    }
                    drop(text);
                    loro_b.import(loro.export(loro_b.vv()));
                }
            })
        });
    }
}
pub fn dumb(_c: &mut Criterion) {}

#[cfg(feature = "fuzzing")]
criterion_group!(benches, run::two_client_edits, run::b4);
#[cfg(not(feature = "fuzzing"))]
criterion_group!(benches, dumb);
criterion_main!(benches);
