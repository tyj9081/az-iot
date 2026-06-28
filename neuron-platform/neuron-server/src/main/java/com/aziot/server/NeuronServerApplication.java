package com.aziot.server;

import org.mybatis.spring.annotation.MapperScan;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.scheduling.annotation.EnableScheduling;

@SpringBootApplication
@ComponentScan("com.aziot")
@MapperScan("com.aziot.dao.mapper")
@EnableScheduling
public class NeuronServerApplication {
    public static void main(String[] args) {
        SpringApplication.run(NeuronServerApplication.class, args);
    }
}
