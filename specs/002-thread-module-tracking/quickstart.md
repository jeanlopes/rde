# Quick Start — Thread and Module Tracking

> Guia rápido dos novos comandos de threads e módulos no RDE.

---

## Comandos de Threads

### Listar threads

```
rde> threads
 ID       Estado      Selecionada
 1234     Running     *
 5678     Suspended
 9012     Running
```

A thread marcada com `*` é a thread selecionada atualmente.

### Selecionar uma thread

```
rde> thread 5678
Thread 5678 selecionada.
```

### Inspecionar registradores da thread selecionada

```
rde> regs
RAX: 0000000000000000  RBX: 0000000000000000
...
```

O comando `regs` sempre opera na thread selecionada.

### Stack trace da thread selecionada

```
rde> bt
#0 some_function    em app.rs:42
#1 caller           em app.rs:30
```

---

## Comandos de Módulos

### Listar módulos carregados

```
rde> modules
 Nome                Base              Tamanho    Símbolos
 hello_debuggee.exe  0x00007FF612340000  0x25000    ✓
 ntdll.dll           0x00007FFEEABC0000  0x1F0000   ✓
 kernel32.dll        0x00007FFEEB000000  0x9F000    ✓
 ucrtbase.dll        0x00007FFEE9000000  0x120000   ✗
```

### Observar carregamento dinâmico

Enquanto o programa roda, novas DLLs aparecem automaticamente:

```
rde> continue
Executando...
[Módulo carregado] my_plugin.dll em 0x00007FF6123A0000
```

---

## Notificações Automáticas

O REPL exibe mensagens quando eventos de thread/módulo ocorrem:

```
[Thread criada] TID 3456
[Thread encerrada] TID 3456 (código: 0)
[Módulo carregado] new_dll.dll em 0x00007FF6123B0000
[Módulo descarregado] 0x00007FF6123B0000
```
