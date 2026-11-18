package bo.edu.usfx.smartfila.ventas.infraestructura.mensajeria;

import com.rabbitmq.client.AMQP;
import com.rabbitmq.client.BuiltinExchangeType;
import com.rabbitmq.client.Channel;
import com.rabbitmq.client.Connection;
import com.rabbitmq.client.ConnectionFactory;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import java.nio.charset.StandardCharsets;
import java.util.UUID;

@Component
public class RabbitMqEventPublisher {

    private static final Logger log = LoggerFactory.getLogger(RabbitMqEventPublisher.class);

    private final boolean enabled;
    private final String host;
    private final int port;
    private final String username;
    private final String password;
    private final String exchange;

    public RabbitMqEventPublisher(
            @Value("${smartfila.rabbitmq.enabled:false}") boolean enabled,
            @Value("${smartfila.rabbitmq.host:rabbitmq}") String host,
            @Value("${smartfila.rabbitmq.port:5672}") int port,
            @Value("${smartfila.rabbitmq.username:smartfila}") String username,
            @Value("${smartfila.rabbitmq.password:cambiar-esta-clave}") String password,
            @Value("${smartfila.rabbitmq.exchange:smartfila.domain}") String exchange
    ) {
        this.enabled = enabled;
        this.host = host;
        this.port = port;
        this.username = username;
        this.password = password;
        this.exchange = exchange;
    }

    public boolean publicar(UUID eventId, String routingKey, String payload, String correlationId) {
        if (!enabled) {
            return false;
        }

        ConnectionFactory factory = new ConnectionFactory();
        factory.setHost(host);
        factory.setPort(port);
        factory.setUsername(username);
        factory.setPassword(password);
        factory.setConnectionTimeout(3000);

        try (Connection connection = factory.newConnection();
             Channel channel = connection.createChannel()) {
            channel.exchangeDeclare(exchange, BuiltinExchangeType.TOPIC, true);
            channel.confirmSelect();

            AMQP.BasicProperties props = new AMQP.BasicProperties.Builder()
                    .deliveryMode(2)
                    .contentType("application/json")
                    .contentEncoding("UTF-8")
                    .messageId(eventId.toString())
                    .correlationId(correlationId)
                    .build();

            channel.basicPublish(
                    exchange,
                    routingKey,
                    props,
                    payload.getBytes(StandardCharsets.UTF_8)
            );
            channel.waitForConfirmsOrDie(5000);

            log.info("Evento {} publicado en {} con routingKey={} correlationId={}",
                    eventId, exchange, routingKey, correlationId);
            return true;
        } catch (Exception exception) {
            log.warn("No se pudo publicar el evento {} en RabbitMQ (exchange={}, routingKey={}): {}",
                    eventId, exchange, routingKey, exception.getMessage());
            return false;
        }
    }
}
