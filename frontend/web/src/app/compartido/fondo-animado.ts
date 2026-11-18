import {
  ChangeDetectionStrategy,
  Component,
  DestroyRef,
  ElementRef,
  ViewEncapsulation,
  afterNextRender,
  inject,
  input,
  viewChild,
} from '@angular/core';
import { EVENTO_DESTACADO } from '../core/eventos';

const SVG = 'http://www.w3.org/2000/svg';

interface Calle {
  d: string;
  /** Segundos que tarda una persona en recorrer la calle. */
  segundos: number;
  /** Número del primer puesto de la calle. */
  desde: number;
}

// Lienzo de 1440×900 que cubre toda la pantalla (xMidYMid slice): una calle
// cruza el fondo por arriba y otra por abajo, enmarcando el contenido.
const CALLES: Calle[] = [
  {
    d: 'M -160 160 C 220 40, 560 180, 900 92 S 1340 0, 1620 56',
    segundos: 60,
    desde: 1,
  },
  {
    d: 'M -160 862 C 260 780, 620 930, 960 836 S 1330 724, 1620 784',
    segundos: 58,
    desde: 201,
  },
];

/** Cada calle del fondo lleva el nombre de una calle con venta del evento destacado. */
const NOMBRES = Object.values(EVENTO_DESTACADO.tramos ?? {}).map((t) => t.calle);
/** Puntos del recorrido (0–1) donde se repite el nombre de la calle. */
const ROTULOS = [0.12, 0.47, 0.82];

const PASO = 34; // distancia entre puestos
const SEPARACION = 36; // del centro de la calle al centro del puesto
const TAM = 22; // lado del puesto
const GRUPOS = 3;
const POR_GRUPO = 7;

/**
 * Fondo de la aplicación, el sistema visto desde arriba:
 * - las calles de la Entrada con puestos numerados en ambas aceras
 *   (impares a la izquierda, pares a la derecha, como las casas);
 * - gente que avanza en fila por la calle (la primera, en dorado, ya tiene turno);
 * - los puestos se van vendiendo en vivo.
 * Con "reducir movimiento" queda quieto.
 */
@Component({
  selector: 'app-fondo-animado',
  changeDetection: ChangeDetectionStrategy.OnPush,
  // Sin encapsulación: los nodos se crean por código y no llevan los atributos de Angular.
  encapsulation: ViewEncapsulation.None,
  host: { 'aria-hidden': 'true', '[class.suave]': 'suave()' },
  template: `
    <div class="fondo-rejilla"></div>
    <svg class="fondo-lienzo" viewBox="0 0 1440 900" preserveAspectRatio="xMidYMid slice">
      <g #lienzo></g>
    </svg>
  `,
  styles: `
    app-fondo-animado {
      position: fixed;
      inset: 0;
      z-index: 0;
      overflow: hidden;
      pointer-events: none;
    }
    .fondo-rejilla {
      position: absolute;
      inset: 0;
      background-image:
        linear-gradient(var(--rejilla) 1px, transparent 1px),
        linear-gradient(90deg, var(--rejilla) 1px, transparent 1px);
      background-size: 56px 56px;
      mask-image: radial-gradient(ellipse 80% 70% at 50% 45%, #000 30%, transparent 80%);
    }
    .fondo-lienzo {
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      transition: opacity 0.6s ease;
    }
    /* Con sesión: las calles se ven a los costados y casi nada detrás de la
       columna de contenido (1120 px), para que títulos y tablas se lean. */
    app-fondo-animado.suave .fondo-lienzo {
      opacity: 0.75;
      --despejado: linear-gradient(
        90deg,
        #000,
        #000 calc(50% - 700px),
        rgb(0 0 0 / 7%) calc(50% - 590px),
        rgb(0 0 0 / 7%) calc(50% + 590px),
        #000 calc(50% + 700px),
        #000
      );
      -webkit-mask-image: var(--despejado);
      mask-image: var(--despejado);
    }
    .fondo-calle {
      fill: none;
      stroke: var(--calle);
      stroke-width: 46;
      stroke-linecap: round;
    }
    .fondo-puesto rect {
      fill: var(--superficie);
      stroke: var(--puesto-borde);
      stroke-width: 1.5;
      transform-box: fill-box;
      transform-origin: center;
      transition:
        fill 0.6s ease,
        stroke 0.6s ease;
    }
    .fondo-puesto text {
      fill: var(--puesto-numero);
      font:
        700 9px/1 'Plus Jakarta Sans Variable',
        system-ui,
        sans-serif;
      text-anchor: middle;
      dominant-baseline: central;
      transition: fill 0.6s ease;
    }
    .fondo-puesto.vendido rect {
      fill: var(--puesto-vendido);
      stroke: var(--puesto-vendido);
    }
    .fondo-puesto.vendido text {
      fill: var(--sobre-primario);
    }
    .fondo-puesto.recien rect {
      animation: fondo-ocupar 0.7s cubic-bezier(0.3, 1.6, 0.5, 1);
    }
    .fondo-nombre {
      fill: var(--puesto-numero);
      font:
        700 8.5px/1 'Plus Jakarta Sans Variable',
        system-ui,
        sans-serif;
      letter-spacing: 0.22em;
      word-spacing: 0.3em;
    }
    .fondo-persona {
      fill: var(--persona);
    }
    .fondo-persona.frente {
      fill: var(--acento);
    }
    @keyframes fondo-ocupar {
      from {
        transform: scale(0.3);
      }
    }
  `,
})
export class FondoAnimado {
  /** Menos intenso detrás de tablas y formularios largos. */
  readonly suave = input(false);

  private readonly lienzo = viewChild.required<ElementRef<SVGGElement>>('lienzo');
  private puestos: SVGGElement[] = [];
  private intervalo: ReturnType<typeof setInterval> | undefined;

  constructor() {
    const destroyRef = inject(DestroyRef);
    afterNextRender(() => {
      const consulta =
        typeof matchMedia === 'function' ? matchMedia('(prefers-reduced-motion: reduce)') : null;
      const dibujar = () => this.dibujar(!consulta?.matches);
      dibujar();
      consulta?.addEventListener('change', dibujar);
      destroyRef.onDestroy(() => {
        consulta?.removeEventListener('change', dibujar);
        clearInterval(this.intervalo);
      });
    });
  }

  private dibujar(conMovimiento: boolean): void {
    clearInterval(this.intervalo);
    const lienzo = this.lienzo().nativeElement;
    lienzo.replaceChildren();
    this.puestos = [];

    CALLES.forEach((calle, i) => {
      const id = `fondo-calle-${i}`;
      const via = crear('path', { id, d: calle.d, class: 'fondo-calle' }) as SVGPathElement;
      lienzo.append(via);
      const nombre = NOMBRES[i];
      if (nombre) this.rotular(lienzo, id, nombre);
      this.dibujarPuestos(lienzo, via, calle);
      if (conMovimiento) this.dibujarFila(lienzo, calle);
    });

    if (conMovimiento) this.intervalo = setInterval(() => this.venderUno(), 700);
  }

  /** Nombre de la calle sobre la calzada, entre la fila y la acera derecha. */
  private rotular(lienzo: SVGGElement, id: string, nombre: string): void {
    for (const punto of ROTULOS) {
      const texto = crear('text', { class: 'fondo-nombre', dy: 17 });
      const camino = crear('textPath', { href: `#${id}`, startOffset: `${punto * 100}%` });
      camino.textContent = nombre.toUpperCase();
      texto.append(camino);
      lienzo.append(texto);
    }
  }

  /** Puestos numerados a lo largo de las dos aceras de la calle. */
  private dibujarPuestos(lienzo: SVGGElement, via: SVGPathElement, calle: Calle): void {
    // jsdom (pruebas) no implementa la geometría de SVG.
    if (typeof via.getTotalLength !== 'function') return;
    const largo = via.getTotalLength();
    let numero = calle.desde;
    for (let s = PASO / 2; s < largo; s += PASO) {
      const p = via.getPointAtLength(s);
      const q = via.getPointAtLength(Math.min(s + 1, largo));
      const angulo = Math.atan2(q.y - p.y, q.x - p.x);
      const grados = ((angulo * 180) / Math.PI).toFixed(1);
      // -1: acera izquierda (impares); 1: acera derecha (pares).
      for (const lado of [-1, 1]) {
        const x = p.x - Math.sin(angulo) * SEPARACION * lado;
        const y = p.y + Math.cos(angulo) * SEPARACION * lado;
        const puesto = crear('g', {
          class: Math.random() < 0.35 ? 'fondo-puesto vendido' : 'fondo-puesto',
          transform: `translate(${x.toFixed(1)} ${y.toFixed(1)}) rotate(${grados})`,
        }) as SVGGElement;
        const texto = crear('text', {});
        texto.textContent = String(numero++);
        puesto.append(
          crear('rect', { x: -TAM / 2, y: -TAM / 2, width: TAM, height: TAM, rx: 4 }),
          texto,
        );
        lienzo.append(puesto);
        this.puestos.push(puesto);
      }
    }
  }

  /** Grupos de personas caminando en fila por el centro de la calle. */
  private dibujarFila(lienzo: SVGGElement, calle: Calle): void {
    for (let g = 0; g < GRUPOS; g++) {
      for (let k = 0; k < POR_GRUPO; k++) {
        // Más desfase = más adelantada en el recorrido: la última del bucle va primera.
        const desfase = (g * calle.segundos) / GRUPOS + k * 1.05;
        const tiempo = {
          dur: `${calle.segundos}s`,
          begin: `-${desfase.toFixed(2)}s`,
          repeatCount: 'indefinite',
        };
        const persona = crear('g', {
          class: k === POR_GRUPO - 1 ? 'fondo-persona frente' : 'fondo-persona',
          opacity: '0',
        });
        // SMIL lee begin/dur/path al insertar: por eso todo se fija antes de agregar al DOM.
        persona.append(
          crear('circle', { cx: 0, cy: -4.2, r: 3.1 }),
          crear('path', { d: 'M -5 7 C -5 0.6, 5 0.6, 5 7 Z' }),
          crear('animateMotion', { ...tiempo, path: calle.d }),
          crear('animate', {
            ...tiempo,
            attributeName: 'opacity',
            values: '0;1;1;0',
            keyTimes: '0;0.05;0.95;1',
          }),
        );
        lienzo.append(persona);
      }
    }
  }

  /** Vende un puesto visible; si hay muchos vendidos, libera algunos. */
  private venderUno(): void {
    const visibles = this.puestos.filter((p) => {
      const r = p.getBoundingClientRect();
      return r.right > 0 && r.left < innerWidth && r.bottom > 0 && r.top < innerHeight;
    });
    const vendidos = visibles.filter((p) => p.classList.contains('vendido'));
    if (vendidos.length > visibles.length * 0.65) {
      for (let i = 0; i < 4; i++) azar(vendidos)?.classList.remove('vendido');
    }
    const libre = azar(visibles.filter((p) => !p.classList.contains('vendido')));
    if (!libre) return;
    libre.classList.add('vendido', 'recien');
    setTimeout(() => libre.classList.remove('recien'), 800);
  }
}

function crear(etiqueta: string, atributos: Record<string, string | number>): SVGElement {
  const el = document.createElementNS(SVG, etiqueta);
  for (const [k, v] of Object.entries(atributos)) el.setAttribute(k, String(v));
  return el;
}

function azar<T>(lista: T[]): T | undefined {
  return lista[Math.floor(Math.random() * lista.length)];
}
