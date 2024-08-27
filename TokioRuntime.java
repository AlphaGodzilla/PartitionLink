public class TokioRuntime {
    static {
        System.loadLibrary("PartitionLink");
    }

    public static native void start();

    public static void main(String[] args) {
        System.out.println("准备启动");
        TokioRuntime.start();
        System.out.println("结束");
    }
}
