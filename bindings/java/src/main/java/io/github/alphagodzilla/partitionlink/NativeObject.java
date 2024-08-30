package io.github.alphagodzilla.partitionlink;

import java.util.concurrent.atomic.AtomicBoolean;

import lombok.Getter;

public abstract class NativeObject implements AutoCloseable {
    @Getter
    private final long nativeObjPtr;

    private final AtomicBoolean disposed = new AtomicBoolean(false);

    static {
        LibLoader.load();
    }

    public NativeObject(long nativeObjPtr) {
        this.nativeObjPtr = nativeObjPtr;
    }

    @Override
    public void close() throws Exception {
        if (disposed.compareAndSet(false, true)) {
            disppose(nativeObjPtr);
        }
    }

    public boolean disposed() {
        return disposed.get();
    }

    protected abstract void disppose(long objPtr);
}
