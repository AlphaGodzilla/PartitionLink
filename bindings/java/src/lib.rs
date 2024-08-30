use jni::objects::{JClass, JString};
use jni::sys::JNIEnv;
use PartitionLink::db::Database;
// use PartitionLink::db::Database;

#[no_mangle]
pub extern "C" fn Java_io_github_alphagodzilla_partitionlink_AsyncDatabase_newDB(mut env: JNIEnv, _class: JClass) -> jlong {
    Box::into_raw(Box::new(Database::default())) as jlong
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_github_alphagodzilla_partitionlink_AsyncDatabase_disppose(
    mut env: JNIEnv,
    _class: JClass,
    db: *mut Database,
) {
    drop(Box::from_raw(db))
}

pub extern "C" fn Java_io_github_alphagodzilla_partitionlink_AsyncDatabase_setStr(
    mut env: JNIEnv,
    _class: JClass,
    db: *mut Database,
    key: JString,
    value: JString,
) {
    db.str_set(key,);
}