pub fn generate_offline_uuid(name: String) -> String {
    let digest = md5::compute(format!("OfflinePlayer:{}", name)).0;
    uuid::Builder::from_md5_bytes(digest).into_uuid().simple().to_string()
}