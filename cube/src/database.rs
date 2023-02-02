#[cfg(not(target_arch = "wasm32"))]
fn get_asanas_from_db() -> Vec<AsanaDB> {
    let path = "./yogamatdb.sql";
    let db = Connection::open(path).expect("couldn't open database");
    //let sql = "select asanaID, sanskritName, englishName, userNotes from asana";
    let sql = r#"
SELECT a.poseId, a.asanaID, b.sanskritName, b.englishName, b.userNotes
FROM pose a, asana b
WHERE a.asanaID = b.asanaID;
"#;
    let mut stmt = db.prepare(&sql).expect("trouble preparing statement");
    let response = stmt
        .query_map([], |row| {
            Ok(AsanaDB {
                pose_id: row.get(0).expect("poseId"),
                asana_id: row.get(1).expect("asanaID"),
                sanskrit: row.get(2).expect("sanskritName"),
                english: row.get(3).expect("englishName"),
                notes: row.get(4).expect("userNotes"),
            })
        })
        .expect("bad");
    let asanas = response
        .filter_map(|result| result.ok())
        .collect::<Vec<AsanaDB>>();

    asanas
}

#[cfg(not(target_arch = "wasm32"))]
fn serialize_db() {
    let asanas = get_asanas_from_db();
    let mut data = AsanaData {
        asanas,
        poses: HashMap::new(),
    };

    let path = "./yogamatdb.sql";
    let db = Connection::open(path).expect("couldn't open database");

    for asana in data.asanas.iter() {
        let sql = format!("select * from joint where poseID = {};", asana.pose_id);
        let mut stmt = db.prepare(&sql).expect("trouble preparing statement");
        let response = stmt
            .query_map([], |row| {
                Ok(Joint {
                    joint_id: row.get(0).expect("so may results"),
                    pose_id: row.get(1).expect("so may results"),
                    up_x: row.get(2).expect("so may results"),
                    up_y: row.get(3).expect("so may results"),
                    up_z: row.get(4).expect("so may results"),
                    forward_x: row.get(5).expect("so may results"),
                    forward_y: row.get(6).expect("so may results"),
                    forward_z: row.get(7).expect("so may results"),
                    origin_x: row.get(8).expect("so may results"),
                    origin_y: row.get(9).expect("so may results"),
                    origin_z: row.get(10).expect("so may results"),
                })
            })
            .expect("bad");
        let joints = response
            .filter_map(|result| result.ok())
            .collect::<Vec<Joint>>();
        // do we store the matrices instead of these joints at some point in the future?
        /*let matrices = joints.iter().map(|joint| JointMatrix {
            mat: joint.matrix(),
            joint_id: joint.joint_id,
        }).collect::<Vec<JointMatrix>>();*/
        let already = data.poses.insert(asana.pose_id, joints);
        assert!(already.is_none());
    }

    let encoded: Vec<u8> = bincode::serialize(&data).unwrap();
    let mut out_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open("out_db")
        .expect("file couldn't be opened");
    let success = out_file.write_all(&encoded);
    match success {
        Ok(_) => println!("encoded db written to out_db"),
        Err(_) => panic!("encoded db file write failed"),
    }
}

