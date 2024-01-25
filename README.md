# JRM

A very simple jvm written in rust which can only run "Hello World".

```java
public class Main {
    private int z;
    public int add(int x, int y) {
        return x + y + z;
    }

    Main(int z) {
        this.z = z;
    }

    public static void main(String[] args) {
        System.out.println("Hello, World");
        int x = 5;
        int y = 68;
        Main m = new Main(10);
        int z = m.add(x, y);
        System.out.println(z);
    }
}
```

## Usage

```bash
cargo run --release -- ./Main.class
```

## Very Simple GC

It's a very simple GC which only collect objects which are not referenced by any other objects after each method call.

```bash
cargo run --release -- ./GC.class
```
