package io.datawire.labs.hellospring;

import java.util.concurrent.TimeUnit;

import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class HelloController {

    private static long start = System.currentTimeMillis();

    @GetMapping("/")
    public String sayHello() {
        long millis = System.currentTimeMillis() - start;
        String uptime = String.format("%02d:%02d",
                                      TimeUnit.MILLISECONDS.toMinutes(millis),
                                      TimeUnit.MILLISECONDS.toSeconds(millis) -
                                      TimeUnit.MINUTES.toSeconds(TimeUnit.MILLISECONDS.toMinutes(millis)));
        return String.format("Hello, Spring! (up %s, %s)", uptime, System.getenv("BUILD_PROFILE"));
    }

}
