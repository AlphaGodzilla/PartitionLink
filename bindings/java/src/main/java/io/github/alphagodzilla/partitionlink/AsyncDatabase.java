package io.github.alphagodzilla.partitionlink;

public class AsyncDatabase extends NativeObject {
    public AsyncDatabase(long nativeObjPtr) {
        super(nativeObjPtr);
    }

    public void setStr(String key, String value) {

    }

    public String getStr(String key) {

    }

    @Override
    protected native void disppose(long objPtr);

    protected static native long newDB();
}
