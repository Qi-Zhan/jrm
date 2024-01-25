// A very simple example to have fun with garbage collection.

public class GC {
    private GC inner;

    public void assign(GC m) {
        GC m1 = new GC();
        GC m2 = new GC();
        m.inner = m1;
        // garbage collect m2
    }

    public void new_m() {
        GC m = new GC();
        assign(m);
        // garbage collect m and m1
    }

    public static void main(String[] args) {
        GC m = new GC();
        m.new_m();
    }
}
