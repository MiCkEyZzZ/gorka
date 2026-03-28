# Gorka Format V1

Gorka — бинарный формат сжатия телеметрии GLONASS.

Версия v1 — это простой, быстрый и доменно-оптимизированный codec:

- chunk-based структура
- первый sample без сжатия
- отсальные — через delta-of-delta и XOR
- учёт FDMA через doppler baseline
