# Native Shell parity — user feedback plan

Estado: en curso. Fuente: capturas y notas del usuario comparando con Dioxus.

## Diálogos y formularios

- [x] **Logo search no devuelve resultados.** Al buscar `spotify` el diálogo se queda sin resultados; comprobar llamada al backend, fallback y estados de carga/error, y pintar la cuadrícula/paginación.
- [ ] **Panel “Create new account” mal compuesto.** (mejorado: panel con padding; revisar captura) El formulario inline se solapa con el campo y los botones; debe quedar dentro del desplegable con separación y sin tapar el campo.
- [x] **Dos desplegables abiertos a la vez.** Account y Category pueden abrirse simultáneamente y solaparse. Solo un select debe estar abierto; al abrir uno se cierra el otro.
- [x] **Click fuera de un diálogo debe cerrarlo.** El backdrop debe cerrar el diálogo (sin cerrar al pulsar dentro).
- [x] **Billing schedule desalineado.** Fila “Every 1 month”, Select all/Clear y el grid de días deben alinearse como en Dioxus.
- [x] **Desplegable de moneda demasiado alto y solapado.** Debe tener altura máxima, scroll y abrir/cerrar sin tapar el resto del formulario.
- [x] **Notas solapadas.** La etiqueta `Notes` y el placeholder del textarea se pisan; corregir el textarea compartido y usarlo en todos los formularios.
- [x] **Date picker distinto a Dioxus.** Revisar cabecera, primer día de semana, tamaño, Clear/Today y alineación con el campo de fecha.
- [x] **Restore de backup arrastrable.** El drop zone debe aceptar drag & drop de un `.json`, no solo el selector de archivos.

## Toolbars, menús y listas

- [x] **Borde del grupo grid/list.** Quitar el contenedor con borde/fondo y el anillo del botón activo; dejar los dos iconos como en Dioxus.
- [x] **Posición del menú Filters.** Debe abrirse anclado al botón Filters, no pegado al borde derecho.
- [x] **Posición del menú Sort.** Debe abrirse anclado al botón Sort.
- [x] **Bulk actions no coinciden con Dioxus.** Dioxus usa una barra inline bajo el toolbar (contador, Select all, categoría, activar/pausar, borrar, limpiar), no una pantalla/modal. Rehacerlo inline.
- [x] **Transactions grid y list se ven igual.** Implementar la cuadrícula real de tarjetas y diferenciarla de la lista.
- [x] **Menú Export mal colocado.** Debe abrirse bajo el botón Export, no flotando sobre otra zona.
- [ ] **Filtros que no filtran.** Revisar que cada filtro (estado, frecuencia, categoría, precio) actualice la lista y que el contador sea correcto.

## Navegación y creación

- [x] **Add del calendario no debe cambiar de pantalla.** Debe abrir el diálogo de suscripción sobre el calendario con la fecha elegida.
- [x] **Add del dashboard debe abrir el diálogo en el dashboard.** Las tres acciones (suscripción, transacción, deuda) deben abrir su diálogo in-place sin navegar.
- [x] **Bug: Subscriptions abre el diálogo solo.** A veces al entrar en Subscriptions el formulario ya está abierto por un intent antiguo; consumir/limpiar el intent una sola vez.

## Dashboard

- [x] **Espaciado del primer upcoming payment.** El primer pago tiene un gap distinto al resto; igualar la lista.
- [ ] **Color secundario distinto.** Revisar el color de la tarjeta Pending/secondary frente a Dioxus.

## Reports

- [x] **Donut no hovereable.** Añadir hover/estado interactivo como Dioxus.
- [x] **Ocultar eje Y con privacy mode.** Los importes 0/76/152/228/303 deben ocultarse cuando “hidden amounts” está activo.

## Notifications / delivery

- [x] **Timezone search no filtra.** Al escribir en el selector de timezone debe filtrar la lista.
- [x] **Delivery: botón y toggle incorrectos.** En ntfy debe decir “Save ntfy” (no “Save Email”), y el toggle debe poder activarse/persistir.
