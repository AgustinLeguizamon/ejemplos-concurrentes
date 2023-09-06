use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader};
use std::process;

fn main() {
    /*if let Err(err) = test_csv_2() {
        println!("{}", err);
        process::exit(1);
    }*/
    read_csv_seq();
}

fn test_csv() -> Result<(), Box<dyn Error>> {
    let file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/archive/CAvideos.csv"))?;
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.records() {
        let record = result?;
        let channel_title = &record[3];
        let views: i32 = record[7].parse()?;
        println!("{:?} {}", channel_title, views);
    }
    Ok(())
}

fn test_csv_2() -> Result<(), Box<dyn Error>> {
    let file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/archive/KRvideos.csv"))?;
    let mut reader = csv::Reader::from_reader(file);
    let mut views_per_channel = HashMap::new();
    for result in reader.records() {
        let record = result.ok();
        if record.is_none() {
            continue
        }
        let urecord = record.unwrap();
        let channel_title = String::from(&urecord[3]);
        let views: i32 = urecord[7].parse().unwrap();
        *views_per_channel.entry(channel_title).or_insert(0) += views;
    }
    println!("{:?}", views_per_channel);
    Ok(())
}


fn read_csv_seq_std() {
    let result = read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/archive")).unwrap()
        .map(|dir_entry| dir_entry.unwrap().path())
        .filter(|path| {
            let result = path.extension().unwrap() == "csv";
            println!("Filtering file: {:?}, result {:?}", path, result);
            return result;
        })
        .flat_map(|path| {
            let file = File::open(path);
            let reader = BufReader::new(file.unwrap());
            reader.lines()
        })
        .map(|l| {
            let record = l.unwrap();
            let values: Vec<&str> = record.split(',').collect();
            let channel_title = String::from(values[3]);
            let views: i32 = values[7].parse().unwrap();
            let mut views_per_channel = HashMap::new();
            *views_per_channel.entry(channel_title).or_insert(0) += views;
            views_per_channel
    }).fold(HashMap::new(), |mut acc, views_per_channel| {
        views_per_channel.iter().for_each(|(k, v)| *acc.entry(k.clone()).or_insert(0) += v);
        acc
    });

    println!("{:?}", result)
}

fn read_csv_seq() {
    let result = read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/archive")).unwrap()
        .map(|dir_entry| dir_entry.unwrap().path())
        .filter(|path| {
            let result = path.extension().unwrap() == "csv";
            // println!("Filtering file: {:?}, result {:?}", path, result);
            return result;
        })
        .map(|path| {
            // println!("Opening file: {:?}", path);
            let file = File::open(path);
            let mut reader = csv::Reader::from_reader(file.unwrap());
            let mut views_per_channel = HashMap::new();
            for result in reader.records() {
                let record = result.ok();
                if record.is_none() {
                    continue
                }
                let u_record = record.unwrap();
                let channel_title = String::from(&u_record[3]);
                let views: i64 = u_record[7].parse().unwrap();
                *views_per_channel.entry(channel_title).or_insert(0) += views;
            }
            // println!("{:?}", views_per_channel);
            views_per_channel
    }).fold(HashMap::new(), |mut acc, views_per_channel| {
        views_per_channel.iter().for_each(|(k, v)| *acc.entry(k.clone()).or_insert(0) += v);
        acc
    });

    println!("{:?}", result);

    println!("Canales {:?}", result.keys().count())

}
