package bo.edu.usfx.smartfila.ventas.config;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Profile;
import org.springframework.http.HttpMethod;
import org.springframework.security.config.annotation.web.builders.HttpSecurity;
import org.springframework.security.config.annotation.web.configuration.EnableWebSecurity;
import org.springframework.security.config.http.SessionCreationPolicy;
import org.springframework.security.oauth2.core.DelegatingOAuth2TokenValidator;
import org.springframework.security.oauth2.core.OAuth2Error;
import org.springframework.security.oauth2.core.OAuth2TokenValidatorResult;
import org.springframework.security.oauth2.core.OAuth2TokenValidator;
import org.springframework.security.oauth2.jwt.Jwt;
import org.springframework.security.oauth2.jwt.JwtDecoder;
import org.springframework.security.oauth2.jwt.JwtValidators;
import org.springframework.security.oauth2.jwt.NimbusJwtDecoder;
import org.springframework.security.oauth2.server.resource.authentication.JwtAuthenticationConverter;
import org.springframework.security.oauth2.server.resource.authentication.JwtGrantedAuthoritiesConverter;
import org.springframework.security.web.SecurityFilterChain;

@Configuration
@EnableWebSecurity
public class SecurityConfig {

    @Bean
    @Profile("!test")
    SecurityFilterChain apiSecurity(
            HttpSecurity http,
            JwtAuthenticationConverter jwtAuthenticationConverter
    ) throws Exception {
        return http
                .csrf(csrf -> csrf.disable())
                .sessionManagement(session -> session
                        .sessionCreationPolicy(SessionCreationPolicy.STATELESS))
                .authorizeHttpRequests(authorize -> authorize
                        .requestMatchers("/actuator/health", "/actuator/health/**", "/actuator/info",
                                "/actuator/prometheus",
                                "/v3/api-docs/**", "/swagger-ui/**", "/swagger-ui.html",
                                "/api/v1/ventas/ping")
                        .permitAll()
                        .requestMatchers(HttpMethod.GET, "/api/v1/ventas/*/auditoria")
                        .hasAnyRole("ADMIN", "ORGANIZADOR")
                        .requestMatchers(HttpMethod.GET, "/api/v1/ventas")
                        .hasAnyRole("CIUDADANO", "ADMIN", "ORGANIZADOR")
                        .requestMatchers(HttpMethod.GET, "/api/v1/ventas/referencias-qr/**")
                        .hasAnyRole("CIUDADANO", "INSPECTOR", "ADMIN")
                        .requestMatchers(HttpMethod.GET, "/api/v1/ventas/*/comprobante")
                        .hasAnyRole("CIUDADANO", "ADMIN", "ORGANIZADOR")
                        .requestMatchers(HttpMethod.POST, "/api/v1/ventas/*/anulaciones")
                        .hasRole("ADMIN")
                        .requestMatchers("/api/v1/ventas/verificaciones/**")
                        .hasAnyRole("INSPECTOR", "ADMIN")
                        .requestMatchers(HttpMethod.POST, "/api/v1/ventas",
                                "/api/v1/ventas/*/confirmaciones-pago-simuladas")
                        .hasRole("CIUDADANO")
                        .requestMatchers("/api/v1/ventas/**").authenticated()
                        .anyRequest().denyAll())
                .oauth2ResourceServer(oauth2 -> oauth2.jwt(jwt ->
                        jwt.jwtAuthenticationConverter(jwtAuthenticationConverter)))
                .build();
    }

    @Bean
    @Profile("!test")
    JwtDecoder jwtDecoder(
            @Value("${smartfila.security.jwk-set-uri}") String jwkSetUri,
            @Value("${smartfila.security.issuer}") String issuer,
            @Value("${smartfila.security.audience}") String audience
    ) {
        NimbusJwtDecoder decoder = NimbusJwtDecoder.withJwkSetUri(jwkSetUri).build();
        OAuth2TokenValidator<Jwt> issuerValidator = JwtValidators.createDefaultWithIssuer(issuer);
        OAuth2TokenValidator<Jwt> audienceValidator = token ->
                token.getAudience() != null && token.getAudience().contains(audience)
                        ? OAuth2TokenValidatorResult.success()
                        : OAuth2TokenValidatorResult.failure(new OAuth2Error(
                                "invalid_token", "audiencia inválida", null
                        ));
        decoder.setJwtValidator(new DelegatingOAuth2TokenValidator<>(
                issuerValidator, audienceValidator
        ));
        return decoder;
    }

    @Bean
    @Profile("!test")
    JwtAuthenticationConverter jwtAuthenticationConverter() {
        JwtGrantedAuthoritiesConverter roles = new JwtGrantedAuthoritiesConverter();
        roles.setAuthoritiesClaimName("roles");
        roles.setAuthorityPrefix("ROLE_");
        JwtAuthenticationConverter converter = new JwtAuthenticationConverter();
        converter.setJwtGrantedAuthoritiesConverter(roles);
        return converter;
    }

    @Bean
    @Profile("test")
    SecurityFilterChain testSecurity(HttpSecurity http) throws Exception {
        return http.csrf(csrf -> csrf.disable())
                .authorizeHttpRequests(authorize -> authorize.anyRequest().permitAll())
                .build();
    }
}
