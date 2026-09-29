import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;

public class StdinEcho {
    public static void main(String[] args) throws Exception {
        BufferedReader input = new BufferedReader(
            new InputStreamReader(System.in, StandardCharsets.UTF_8)
        );
        String line = input.readLine();
        System.out.println("received:" + line);
    }
}
