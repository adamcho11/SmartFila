package bo.edu.usfx.smartfila.ventas.config;

import io.micrometer.core.instrument.MeterRegistry;
import org.springframework.stereotype.Component;

@Component
public class VentasMetrics {
    private final MeterRegistry registry;

    public VentasMetrics(MeterRegistry registry) {
        this.registry = registry;
    }

    public void ordenCreada() {
        registry.counter("smartfila.sales.orders", "result", "created").increment();
    }

    public void pagoConfirmado() {
        registry.counter("smartfila.sales.payments", "result", "confirmed").increment();
    }

    public void inspeccion(String metodo, boolean valido) {
        registry.counter("smartfila.sales.inspections", "method", metodo,
                "result", valido ? "valid" : "invalid").increment();
    }
}
