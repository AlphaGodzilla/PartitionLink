public class HelloWorld {
    static {
        System.loadLibrary("PartitionLink");
    }

    private static native String hello(String input);

    public static void main(String[] args) {
        String output = HelloWorld.hello("josh");
        System.out.println(output);
    }
}
