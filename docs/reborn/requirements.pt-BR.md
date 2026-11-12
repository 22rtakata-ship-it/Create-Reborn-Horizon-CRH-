# PROJETO: CREATE REBORN FOR HORIZON

## 1. Objetivo geral

Quero desenvolver uma implementação nativa em Rust, inspirada no mod Create do Minecraft Java, para o Minecraft Horizon.

O objetivo não é simplesmente traduzir o código Java para Rust, mas reconstruir e adaptar as funcionalidades do Create utilizando a arquitetura, o sistema de mods e as possibilidades de otimização do Horizon.

O projeto deverá preservar a essência do Create original:

- Energia mecânica por rotação.
- Engrenagens e transmissão de movimento.
- Rodas d'água e moinhos de vento.
- Automação de recursos.
- Processamento e multiplicação de minérios.
- Máquinas industriais mecânicas.
- Transporte automatizado de itens.
- Contraptions e estruturas móveis.
- Liberdade para construir máquinas personalizadas.
- Progressão survival sem dependência de eletricidade.

O projeto deve ser dividido em três módulos principais:

1. **Create Core**
2. **Create Mechanics**
3. **Create Contraptions**

Esses módulos devem ser desenvolvidos em uma arquitetura modular, organizada e expansível, permitindo adicionar funcionalidades futuramente sem reescrever sistemas já existentes.

---

## 2. Referências obrigatórias e investigação inicial

Antes de escrever código, investigue os seguintes projetos:

### Minecraft Horizon

- https://github.com/horizon-rust/horizon
- https://playhorizon.dev/docs
- https://playhorizon.dev/docs/mods
- https://playhorizon.dev/docs/make-a-mod
- https://github.com/horizon-rust/horizon-mods

Estude:

- Arquitetura do jogo.
- Sistema de mods nativos em Rust.
- Formato `.hnmod`.
- Formato `.hmod`.
- Sistema de mixins e hooks.
- Registro de blocos e itens.
- Block states.
- Block entities.
- Sistema de ticks.
- Renderização de blocos e entidades.
- Manipulação de inventários.
- Colisões e física.
- Serialização e salvamento de mundos.
- Multiplayer.
- Sistema de receitas.
- Integração de modelos e texturas.

### Create original

- https://github.com/Creators-of-Create/Create

Utilize o Create original como referência funcional para compreender:

- Redes cinéticas.
- Propagação de velocidade.
- Stress capacity e stress impact.
- Relações entre engrenagens.
- Processamento de itens.
- Receitas.
- Contraptions.
- Estruturas móveis.
- Movimentação e colisões.
- Renderização de componentes mecânicos.

Use como referência principal uma versão estável do Create 6.x para Minecraft 1.21.1, preferencialmente a versão utilizada pelo projeto Create for Horizon.

Não misture implementações de versões diferentes sem documentar as diferenças.

### Create for Horizon — implementação existente

- https://playhorizon.dev/mods/create

Este projeto é uma implementação experimental do Create em Rust para o Horizon.

Localize o repositório de código-fonte oficial do mod por meio da página acima.

Antes de iniciar uma implementação própria:

1. Analise todo o projeto existente.
2. Identifique as funcionalidades já implementadas.
3. Identifique quais funcionalidades realmente funcionam.
4. Identifique código experimental, incompleto ou provisório.
5. Identifique os hooks utilizados.
6. Verifique as versões compatíveis do Horizon e Horizon Mixins.
7. Verifique as licenças do código e dos recursos.
8. Determine se é mais vantajoso utilizar um fork, reaproveitar componentes ou desenvolver uma implementação independente.

Priorize reaproveitar código funcional, licenciado e compatível, evitando reimplementar sistemas desnecessariamente.

Não considere uma funcionalidade concluída somente porque existe um arquivo ou uma classe com seu nome. Verifique seu comportamento real e seus testes.

Preserve os créditos, arquivos LICENSE, NOTICE e demais obrigações legais dos projetos utilizados.

Não redistribua assets que não possuam licença adequada. Caso necessário, implemente importação local de modelos e texturas a partir de uma instalação legítima do Create.

---

## 3. Arquitetura obrigatória

Organize o projeto em um Cargo workspace com módulos ou crates claramente separados.

Estrutura inicial desejada:

```
create-reborn/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── NOTICE
├── AGENTS.md
├── docs/
│   ├── architecture.md
│   ├── compatibility.md
│   ├── roadmap.md
│   ├── mechanics.md
│   ├── contraptions.md
│   ├── performance.md
│   ├── testing.md
│   └── project-map.md
├── crates/
│   ├── create-core/
│   ├── create-mechanics/
│   ├── create-contraptions/
│   ├── create-horizon-adapter/
│   └── create-testkit/
├── assets/
├── tests/
├── benchmarks/
├── tools/
└── packaging/
```

A estrutura pode ser adaptada caso o sistema real de mods do Horizon exija outro formato, mas a separação lógica dos módulos é obrigatória.

### Dependências

```
Create Core
    ↑
    ├── Create Mechanics
    └── Create Contraptions
```

Regras:

- Create Core não pode depender de Mechanics.
- Create Core não pode depender de Contraptions.
- Mechanics depende de Core.
- Contraptions depende de Core.
- Contraptions deve funcionar sem Mechanics quando possível.
- A integração entre Mechanics e Contraptions deve utilizar interfaces públicas e recursos opcionais.
- Não criar dependências circulares.
- Não espalhar código específico do Horizon entre todos os componentes quando uma camada de adaptação puder resolver.
- Separar regras de simulação da integração com a engine.
- Separar simulação do servidor e renderização do cliente.

Crie um `create-horizon-adapter` responsável por integrar os módulos à API e aos hooks do Horizon.

Caso a arquitetura real do Horizon exija que essa camada seja dividida em vários adaptadores, faça isso e documente a decisão.

Os sistemas principais devem poder ser testados sem inicializar o jogo inteiro.

Não invente APIs, hooks, tipos, métodos ou funcionalidades do Horizon. Confirme a existência de cada integração consultando o código-fonte e a documentação da versão utilizada.

---

# MÓDULO 1 — CREATE CORE

## 4. Responsabilidade do Create Core

O Create Core é a fundação de todo o projeto.

Ele será responsável pelo sistema cinético, geração de energia, transmissão de movimento, gerenciamento de redes mecânicas, interfaces para máquinas e funcionalidades básicas.

Nenhuma máquina de processamento industrial deverá implementar seu próprio sistema de energia.

Todas deverão utilizar o sistema compartilhado do Core.

---

## 5. Sistema de energia cinética

Implemente um sistema centralizado de redes mecânicas.

Cada componente cinético deverá possuir, quando aplicável:

- Identificador do componente.
- Posição no mundo.
- Dimensão.
- Orientação.
- Eixo de rotação.
- Velocidade em RPM.
- Sentido de rotação.
- Stress capacity.
- Stress impact.
- Identificador da rede mecânica.
- Estado de funcionamento.
- Conexões válidas.
- Informações de persistência.

Implemente as seguintes funcionalidades:

### Redes mecânicas

- Detectar componentes conectados.
- Formar redes cinéticas.
- Propagar RPM.
- Propagar sentido de rotação.
- Calcular proporções de engrenagens.
- Calcular geração e consumo de stress.
- Detectar redes sobrecarregadas.
- Detectar fontes incompatíveis.
- Detectar ciclos com relações inconsistentes.
- Unificar redes ao conectar componentes.
- Dividir redes ao remover componentes.
- Atualizar redes quando a orientação de um bloco mudar.
- Atualizar redes quando um chunk carregar ou descarregar.
- Evitar duplicação de energia.
- Evitar propagação infinita.
- Evitar recursão que possa causar stack overflow.

Use uma representação de grafo ou outra estrutura apropriada para as redes.

Não faça uma busca completa em todos os componentes a cada tick.

O sistema deverá utilizar atualizações orientadas por eventos, recalculando a topologia quando houver mudanças reais.

Para cada rede, mantenha um cache dos resultados válidos.

A velocidade e o consumo de stress devem reproduzir o comportamento do Create original sempre que possível.

Não invente valores arbitrários de balanceamento quando os valores de referência puderem ser obtidos no código original.

---

## 6. Componentes mecânicos básicos

Implemente:

### Transmissão

- Shaft.
- Cogwheel.
- Large Cogwheel.
- Encased Shaft.
- Encased Cogwheel.
- Encased Large Cogwheel.
- Gearbox.
- Vertical Gearbox.
- Clutch.
- Gearshift.
- Encased Chain Drive.
- Adjustable Chain Gearshift.
- Rotation Speed Controller.

### Regras de funcionamento

- Engrenagens pequenas e grandes devem possuir relações corretas de transmissão.
- Engrenagens conectadas devem inverter a rotação quando apropriado.
- Eixos devem transmitir movimento somente em direções válidas.
- Gearboxes devem respeitar o eixo e a orientação.
- Clutches devem interromper a transmissão por redstone.
- Gearshifts devem inverter a direção por redstone.
- Controladores devem respeitar as regras de velocidade e stress.
- Componentes incompatíveis não devem conectar incorretamente.
- Redes sobrecarregadas devem parar de funcionar e se recuperar quando o consumo for reduzido.

Crie testes específicos para cada combinação de componente e direção.

---

## 7. Fontes de energia mecânica

Implemente:

### Water Wheel

- Geração de energia a partir de água.
- Detecção das condições válidas de funcionamento.
- Sentido de rotação.
- RPM.
- Stress capacity.
- Animação contínua.
- Interrupção correta ao remover a fonte de água.

### Large Water Wheel

- Comportamento equivalente ao Create original.
- Maior capacidade conforme configuração de referência.
- Renderização e colisões adequadas.

### Hand Crank

- Ativação manual.
- Geração temporária de rotação.
- Comportamento survival.
- Integração com interação do jogador.

### Creative Motor

- Fonte configurável para Creative.
- Ajuste de RPM.
- Utilização como ferramenta de testes.
- Não disponibilizar indevidamente em survival.

### Windmill

O Create Core deverá disponibilizar a interface genérica para fontes de energia de moinhos.

Implemente inicialmente um moinho simples com pás predefinidas somente se isso for necessário para validar o sistema cinético.

A versão completa com montagem personalizada de velas, blocos e estrutura giratória deverá ser implementada pelo Create Contraptions.

Não criar duas implementações incompatíveis de energia e rotação para moinhos.

---

## 8. Recursos básicos e ferramentas

Implemente os recursos fundamentais necessários para a progressão:

- Andesite Alloy.
- Andesite Casing.
- Brass Casing.
- Zinc Ore, quando suportado pela geração de mundo.
- Zinc Ingot.
- Crushed Zinc.
- Brass Ingot.
- Wrench.
- Goggles.
- Super Glue, caso a integração com Contraptions esteja disponível.

Adicione receitas, loot tables, tags, modelos, texturas e traduções.

As receitas e os materiais devem ser configuráveis por dados, evitando regras hardcoded desnecessárias.

Implemente a Wrench para:

- Girar blocos.
- Alterar orientações válidas.
- Ajustar componentes.
- Remover componentes compatíveis conforme as regras do Create.

Implemente as Goggles para exibir:

- RPM.
- Sentido de rotação.
- Stress capacity.
- Stress impact.
- Estado da rede.
- Sobrecarga.
- Informações relevantes da máquina.

---

## 9. API pública do Core

Crie interfaces públicas bem documentadas para:

- Registrar um componente cinético.
- Registrar fontes de energia.
- Registrar consumidores de energia.
- Consultar RPM.
- Consultar direção de rotação.
- Consultar estado de uma rede.
- Consultar stress disponível.
- Alterar consumo de stress.
- Detectar mudanças de topologia.
- Receber eventos de funcionamento.
- Integrar máquinas de outros módulos.
- Registrar contraptions como consumidores ou fontes de energia.

Evite expor diretamente detalhes internos que possam mudar.

Os módulos Mechanics e Contraptions devem conseguir utilizar essa API sem modificar a implementação interna do Core.

---

# MÓDULO 2 — CREATE MECHANICS

## 10. Responsabilidade do Create Mechanics

O Create Mechanics será responsável pela automação industrial, processamento de recursos, transporte de itens e produção em escala.

Todas as máquinas devem utilizar a energia cinética fornecida pelo Core.

A prioridade é reproduzir o gameplay survival do Create, especialmente as linhas de processamento e multiplicação de minérios.

---

## 11. Sistema central de receitas

Crie um sistema genérico de receitas industriais.

Deverá suportar:

- Entradas de itens.
- Quantidades.
- Tags de itens.
- Múltiplos ingredientes.
- Fluidos, quando disponíveis.
- Saídas de itens.
- Subprodutos.
- Resultados probabilísticos.
- Tempo de processamento.
- RPM mínimo, quando aplicável.
- Requisitos de máquina.
- Requisitos de aquecimento.
- Consumo de stress.
- Receitas de processamento contínuo.
- Receitas configuráveis por dados.

Tipos de receita:

- Crushing.
- Milling.
- Pressing.
- Mixing.
- Compacting.
- Washing.
- Splashing.
- Smoking.
- Blasting.
- Haunting.
- Cutting.
- Filling e Emptying, quando houver suporte a fluidos.
- Sequenced Assembly em uma fase posterior.

Não duplique o sistema de receitas para cada máquina.

As máquinas devem utilizar o mesmo sistema central e possuir comportamentos específicos quando necessário.

Receitas devem ser compatíveis com persistência, multiplayer e recarregamento de dados quando a engine permitir.

---

## 12. Crushing Wheels — esmagadores

Implemente Crushing Wheels com prioridade alta.

Funcionalidades:

- Dois esmagadores posicionados corretamente.
- Rotação oposta.
- Detecção de entrada válida.
- Processamento de itens posicionados entre as rodas.
- Receitas de esmagamento.
- Produção de crushed ores.
- Geração de subprodutos.
- Processamento proporcional à velocidade quando aplicável.
- Consumo de stress.
- Integração com esteiras, funis e inventários.
- Prevenção de duplicação ou perda indevida de itens.
- Bloqueio quando as rodas estiverem posicionadas incorretamente.
- Renderização das rodas girando.

Quando possível, reproduza o processamento de entidades e o comportamento de colisão do Create original.

Caso uma funcionalidade física não esteja disponível na engine, documente o bloqueio e implemente primeiro o processamento de itens.

---

## 13. Encased Fan — ventiladores

Implemente Encased Fan como um componente completo de processamento e movimentação de ar.

Funcionalidades:

- Direção do fluxo.
- Sopro.
- Sucção.
- Distância do fluxo conforme velocidade e regras do Create.
- Detecção de obstáculos.
- Interação com itens.
- Interação com entidades, quando disponível.
- Partículas de fluxo.
- Integração com esteiras.
- Processamento em depots.
- Processamento de itens soltos, quando suportado.

Tipos de processamento:

### Bulk Washing

Ventilador com fluxo atravessando água.

- Lavagem de crushed ores.
- Produção de nuggets e subprodutos.
- Receitas de lavagem do Create.

### Bulk Blasting

Ventilador com fluxo atravessando lava ou fonte válida de calor.

- Processamento equivalente às receitas correspondentes.
- Comportamento e resultados fiéis ao Create.

### Bulk Smoking

Ventilador com fluxo atravessando uma fonte válida de fumaça.

- Processamento de alimentos.
- Receitas específicas.

### Bulk Haunting

Ventilador com fluxo atravessando soul fire ou fonte apropriada.

- Transformação de itens conforme receitas de referência.

O sistema deve utilizar receitas configuráveis e não depender de listas fixas espalhadas pelo código.

Não simule todos os blocos do mundo a cada tick para detectar fluxos.

Use regiões afetadas, caches e invalidação localizada.

---

## 14. Multiplicação e processamento de minérios

Este é um dos objetivos principais do projeto.

Implemente uma linha industrial funcional:

```
Minério bruto
    ↓
Crushing Wheels
    ↓
Crushed Ore
    ↓
Esteira transportadora
    ↓
Encased Fan + Água
    ↓
Nuggets e subprodutos
    ↓
Compactação ou crafting
    ↓
Lingotes
```

Adicione cadeias para:

- Iron.
- Gold.
- Copper.
- Zinc.
- Outros minérios relevantes quando suportados.

Reproduza as quantidades, probabilidades e regras da versão de referência do Create.

Não force um multiplicador fixo para todos os minérios. O rendimento deve ser definido pelas receitas correspondentes.

Implemente a linha de processamento tanto manual quanto automatizada.

O jogador deverá conseguir inserir raw ores em uma máquina e coletar os produtos finais automaticamente, utilizando somente energia mecânica.

Crie testes de conservação e transformação de itens que validem:

- Entradas.
- Saídas.
- Subprodutos.
- Resultados probabilísticos.
- Inventários cheios.
- Interrupção de energia.
- Reinício do mundo durante processamento.
- Descarregamento de chunks.
- Prevenção de duplicação.

---

## 15. Máquinas industriais

Implemente progressivamente:

### Mechanical Press

- Prensagem de itens.
- Produção de sheets.
- Processamento em depots.
- Processamento em belts.
- Compactação em Basin.
- Animação da prensa.

### Mechanical Mixer

- Mistura de ingredientes.
- Integração com Basin.
- Produção de materiais como Brass.
- RPM mínimo quando exigido.
- Receitas com aquecimento.
- Animação rotacional e vertical.

### Basin

- Armazenamento de ingredientes.
- Inventário de entrada e saída.
- Receitas de mistura e compactação.
- Integração com fluidos quando disponível.
- Transferência automática.
- Persistência de processamento.

### Millstone

- Moagem.
- Transformação de materiais.
- Receitas configuráveis.
- Integração com inventários e funis.

### Mechanical Drill

- Mineração automatizada.
- Consumo de energia.
- Respeito à dureza do bloco e regras de mineração.
- Geração correta de drops.
- Interrupção em blocos não mineráveis.
- Integração futura com Contraptions.

### Mechanical Saw

- Processamento de madeira.
- Receitas de corte.
- Interação com blocos.
- Integração com contraptions móveis.

### Mechanical Harvester

- Colheita automatizada.
- Respeitar maturidade de plantações.
- Geração correta de produtos e sementes.
- Replantio quando correspondente ao comportamento original.
- Integração com contraptions.

### Mechanical Deployer

- Interações automatizadas.
- Colocação e utilização de itens.
- Processamento de receitas.
- Integração com inventários.
- Validação de permissões e regras do mundo.

### Mechanical Crafter

- Crafting automatizado.
- Receitas com múltiplos componentes.
- Processamento de padrões.
- Consumo de energia.
- Prevenção de loops e duplicações.

### Blaze Burner

- Aquecimento de Basin.
- Estados de calor.
- Combustíveis.
- Compatibilidade com receitas que exigem aquecimento.

A implementação de cada máquina deve possuir lógica funcional, modelos, animações, interação com o jogador e testes.

Não considere uma máquina concluída apenas porque seu bloco pode ser colocado no mundo.

---

## 16. Esteiras e transporte de itens

Implemente um sistema de logística mecânica.

### Mechanical Belt

- Conexão entre eixos.
- Criação de esteiras válidas.
- Transporte conforme RPM.
- Direção de movimentação.
- Transferência de itens.
- Espaçamento entre itens.
- Parada quando não houver energia.
- Sincronização visual.
- Persistência dos itens transportados.
- Suporte a esteiras inclinadas e verticais quando aplicável.
- Integração com máquinas.

Não recrie entidades de item a cada tick apenas para representar itens movimentando-se na esteira.

Implemente uma representação lógica compacta e utilize interpolação visual para o movimento.

### Depot

- Receber itens.
- Armazenar itens.
- Disponibilizar itens para máquinas.
- Receber resultados de processamento.
- Interagir com funis e transportadores.

### Andesite Funnel e Brass Funnel

- Entrada e saída de itens.
- Transferência entre inventários.
- Filtros quando apropriado.
- Limites de transferência.
- Integração com belts.
- Configuração de direção.

### Chute e Smart Chute

- Transporte vertical.
- Transferência gravitacional.
- Filtros.
- Integração com inventários.
- Integração com fluxos de ar quando disponível.

### Brass Tunnel e Andesite Tunnel

- Distribuição de itens.
- Divisão de fluxo.
- Combinação de linhas.
- Filtros e modos de distribuição.

### Mechanical Arm

Implementar posteriormente, após o sistema central de transporte e inventários estar estável.

---

## 17. Fluidos e armazenamento

Após concluir as principais máquinas e a cadeia de processamento de minérios, implementar:

- Fluid Pipe.
- Mechanical Pump.
- Fluid Tank.
- Item Vault.
- Hose Pulley.
- Spout.
- Item Drain.

O sistema de fluidos deverá ser centralizado e possuir API própria.

Implementar:

- Fluxo.
- Pressão ou regras equivalentes do Create.
- Capacidade.
- Tipos de fluidos.
- Transferência.
- Bloqueio de misturas inválidas.
- Persistência.
- Integração com máquinas.

Não bloquear a primeira versão funcional do Mechanics por causa de sistemas de fluidos avançados.

---

# MÓDULO 3 — CREATE CONTRAPTIONS

## 18. Responsabilidade do Create Contraptions

O Create Contraptions será responsável por reproduzir as estruturas móveis do Create original.

Seu objetivo é permitir que o jogador construa estruturas com blocos reais e consiga movimentá-las mecanicamente.

Exemplos:

- Moinhos de vento personalizados.
- Plataformas giratórias.
- Pontes móveis.
- Portões.
- Elevadores.
- Fazendas automáticas giratórias.
- Máquinas móveis de mineração.
- Estruturas movimentadas por pistões.
- Sistemas de transporte móveis.

O módulo deve integrar-se ao Core para utilizar as mesmas redes cinéticas.

---

## 19. Sistema central de contraptions

Implemente uma representação genérica de estruturas móveis.

Cada contraption deverá possuir:

- Identificador persistente.
- Posição no mundo.
- Dimensão.
- Ponto de origem.
- Blocos em coordenadas locais.
- Estados dos blocos.
- Block entity data, quando aplicável.
- Orientação.
- Transformação.
- Velocidade linear.
- Velocidade angular.
- Tipo de movimento.
- Limites espaciais.
- Inventários vinculados.
- Informações de energia.
- Estado de montagem.
- Estado de movimento.
- Dados de persistência.

Implemente sistemas separados para:

- Assembly.
- Disassembly.
- Movement.
- Rotation.
- Collision.
- Rendering.
- Interaction.
- Persistence.
- Synchronization.

A estrutura móvel deve ser um objeto lógico independente dos blocos estáticos do mundo enquanto estiver em movimento.

Não implemente movimentação removendo e recolocando todos os blocos no mundo a cada tick.

---

## 20. Montagem e desmontagem

Implemente:

- Super Glue.
- Linear Chassis.
- Radial Chassis.
- Regras de conexão entre blocos.
- Identificação dos blocos pertencentes à estrutura.
- Limites configuráveis de tamanho.
- Validação de blocos permitidos.
- Validação de obstáculos.
- Detecção de estruturas inválidas.

### Assembly

Ao montar uma contraption:

1. Identificar os blocos conectados.
2. Validar limites e compatibilidade.
3. Capturar blocos, estados e inventários.
4. Criar a estrutura lógica móvel.
5. Remover os blocos correspondentes do mundo de forma segura.
6. Iniciar renderização e simulação móvel.

### Disassembly

Ao desmontar:

1. Validar posição final.
2. Verificar colisões e ocupação.
3. Verificar limites do mundo.
4. Restaurar blocos e estados.
5. Restaurar inventários.
6. Restaurar block entities.
7. Remover a entidade móvel.

A montagem e desmontagem devem ser transacionais.

Se qualquer etapa falhar, a operação deverá ser cancelada ou revertida de forma segura.

Nunca permitir:

- Duplicação de blocos.
- Duplicação de inventários.
- Perda silenciosa de itens.
- Corrupção de chunks.
- Desaparecimento de estruturas após reiniciar o jogo.

---

## 21. Mechanical Bearing

Implemente o Mechanical Bearing como primeiro sistema completo de contraption.

Funcionalidades:

- Montagem de estruturas conectadas.
- Rotação em um eixo.
- Velocidade dependente da rede cinética.
- Mudança de direção.
- Parada.
- Desmontagem em alinhamento válido.
- Renderização contínua.
- Colisão quando suportada.
- Persistência.

Primeiro objetivo funcional:

O jogador deve conseguir construir uma plataforma com blocos comuns, conectá-la a um Mechanical Bearing, alimentar o Bearing com uma Water Wheel e visualizar toda a plataforma girando.

Esse será o teste fundamental da arquitetura de contraptions.

---

## 22. Windmill Bearing e moinhos personalizados

Implemente:

- Windmill Bearing.
- Sail.
- Sail Frame.
- Wool, quando aplicável.
- Detecção dos componentes válidos.
- Montagem de pás personalizadas.
- Cálculo de geração de energia.
- Cálculo de capacidade conforme a estrutura.
- Rotação visual.
- Integração com Core.
- Configuração do sentido de rotação.
- Montagem e desmontagem.

O jogador deverá conseguir construir um moinho com tamanho e formato próprios, semelhante ao Create original.

Não limitar a versão final a um modelo fixo com quatro pás.

O Windmill deverá funcionar como uma fonte de energia mecânica para:

- Engrenagens.
- Eixos.
- Mechanical Press.
- Crushing Wheels.
- Encased Fan.
- Mechanical Belt.
- Demais máquinas do Mechanics.

---

## 23. Movimento linear

Implemente:

### Mechanical Piston

- Extensão.
- Retração.
- Pole extensions.
- Movimento de estruturas conectadas.
- Limites.
- Colisão.
- Mudança de direção.

### Sticky Mechanical Piston

- Comportamento equivalente ao original.
- Captura e movimentação de estruturas compatíveis.

### Rope Pulley

- Elevação.
- Descida.
- Movimento vertical.
- Limites.
- Persistência da estrutura.
- Integração com redstone.

### Gantry Carriage e Gantry Shaft

- Movimento horizontal.
- Velocidade dependente de RPM.
- Mudança de sentido.
- Transporte de estruturas.
- Limites e regras de conexão.

Esses componentes devem reutilizar o mesmo sistema central de contraptions.

Não criar um motor de movimentação completamente diferente para cada máquina.

---

## 24. Controles e automação

Implemente:

- Sequenced Gearshift.
- Controle por redstone.
- Estados de início e parada.
- Alteração de direção.
- Sequências programáveis de rotação.
- Sequências programáveis de deslocamento.
- Integração com Rotation Speed Controller.
- Configuração pelo jogador.

### Elevadores

Após estabilizar o movimento linear:

- Elevator Pulley.
- Elevator Contact.
- Seleção de andares.
- Chamada de plataforma.
- Parada em posições definidas.
- Comportamento seguro diante de obstáculos.
- Persistência da posição e dos destinos.

---

## 25. Interação de máquinas em contraptions

Este é um objetivo essencial do projeto.

Não quero apenas estruturas que se movimentam visualmente. Quero máquinas que funcionem durante o movimento.

Implemente interfaces para atores móveis.

Uma máquina deverá poder possuir dois contextos:

1. Estacionário no mundo.
2. Montado em uma contraption.

Utilize adaptadores para reaproveitar a lógica da máquina sem duplicar seu código.

Implemente integração progressiva de:

- Mechanical Drill.
- Mechanical Saw.
- Mechanical Harvester.
- Mechanical Deployer.
- Inventários móveis.
- Portable Storage Interface.
- Portable Fluid Interface, quando disponível.

Exemplos de funcionamento obrigatório:

### Fazenda giratória

- Mechanical Bearing.
- Estrutura com Mechanical Harvesters.
- Rotação sobre uma plantação.
- Colheita automatizada.
- Replantio quando apropriado.
- Armazenamento dos produtos.
- Transferência para inventário estacionário.

### Máquina móvel de mineração

- Mechanical Piston, Gantry ou sistema móvel compatível.
- Mechanical Drills.
- Mineração conforme a estrutura avança.
- Geração correta de drops.
- Armazenamento em inventários.
- Prevenção de duplicação.

### Ponte móvel

- Estrutura montada.
- Movimento ou rotação.
- Colisões.
- Transporte de jogadores quando suportado.
- Parada e desmontagem seguras.

As máquinas em contraptions não devem depender de serem blocos estáticos registrados diretamente no mundo enquanto estiverem em movimento.

---

## 26. Funcionalidades avançadas

Deixe para uma etapa posterior:

- Cart Assembler.
- Contraptions em minecarts.
- Trens.
- Trilhos personalizados.
- Estações.
- Sinalização ferroviária.
- Controle de tráfego.
- Trens automatizados.
- Contraptions complexas com múltiplos sistemas.
- Ponder completo.

Essas funcionalidades devem estar previstas na arquitetura, mas não devem bloquear o desenvolvimento do Create básico.

---

# REQUISITOS TRANSVERSAIS

## 27. Renderização e desempenho

O projeto deve aproveitar o potencial de desempenho do Rust e do Horizon.

Não implemente os sistemas como uma tradução literal de padrões ineficientes do Java.

Priorize:

### Simulação

- Atualizações orientadas por eventos.
- Redes mecânicas com topologia em cache.
- Dirty flags.
- Atualização localizada.
- Estruturas compactas.
- Evitar alocações desnecessárias.
- Evitar clones de estruturas grandes.
- Evitar buscas globais.
- Evitar processamento repetido.
- Separar dados de simulação e apresentação.
- Atualizar máquinas somente quando necessário.

### Renderização

- Reutilizar modelos.
- Reutilizar malhas.
- Cache de geometria.
- Renderização instanciada quando suportada.
- Batch rendering quando suportado.
- Frustum culling.
- Occlusion culling quando apropriado.
- Interpolação visual.
- Atualização de transformações sem reconstruir malhas.
- Separar geometria estática e móvel.

Engrenagens girando não devem causar reconstrução completa da malha de chunks a cada frame.

As contraptions devem possuir uma malha reutilizável sempre que possível.

A animação de rotação deve utilizar transformações da estrutura, em vez de recriar toda a geometria continuamente.

Caso a renderização do Horizon não ofereça determinado recurso, utilize a melhor alternativa comprovadamente disponível e documente a limitação.

### Multithreading

Utilize processamento paralelo somente onde for seguro e mensuravelmente vantajoso.

Não acesse ou modifique o mundo de forma concorrente sem respeitar as regras da engine.

Não introduza races, deadlocks ou sincronizações excessivas.

Mantenha as atualizações autoritativas do jogo determinísticas quando necessário.

Não use `unsafe` sem justificativa técnica, documentação e testes.

---

## 28. Salvamento, segurança e multiplayer

Implemente persistência para:

- Redes mecânicas ou informações suficientes para reconstruí-las.
- Estados de máquinas.
- Inventários.
- Receitas em andamento.
- Itens em esteiras.
- Fluidos.
- Contraptions montadas.
- Posições.
- Movimentos.
- Configurações do jogador.

Utilize dados versionados e suporte migração quando o formato mudar.

### Multiplayer

Sempre que a arquitetura do Horizon permitir:

- O servidor deve ser autoritativo.
- O cliente não deve decidir resultados de processamento.
- O cliente não deve criar itens.
- O cliente não deve decidir energia disponível.
- Sincronizar estados e transformações.
- Evitar transmissão excessiva de dados.
- Interpolar animações no cliente.
- Validar ações do jogador.
- Garantir compatibilidade de versões entre clientes e servidores.

Priorize inicialmente singleplayer funcional.

Não declare multiplayer suportado até que existam testes com cliente e servidor reais.

---

## 29. Organização do código e eficiência do Codex

Quero que o projeto seja organizado de forma que futuras alterações não exijam analisar todo o código novamente.

Crie um `AGENTS.md` na raiz explicando:

- Objetivo geral do projeto.
- Arquitetura.
- Dependências entre módulos.
- Convenções.
- Comandos de build.
- Comandos de teste.
- Regras de integração.
- Restrições de compatibilidade.
- Processo de implementação.
- Critérios para concluir uma funcionalidade.

Crie arquivos `AGENTS.md` específicos nos módulos quando necessário.

Crie `docs/project-map.md` contendo um índice dos sistemas e seus respectivos arquivos.

Exemplo:

```
Sistema de RPM → create-core/src/kinetics/
Redes mecânicas → create-core/src/network/
Stress → create-core/src/stress/
Fontes de energia → create-core/src/sources/
Crushing Wheels → create-mechanics/src/machines/crushing/
Encased Fan → create-mechanics/src/machines/fan/
Esteiras → create-mechanics/src/logistics/belts/
Receitas → create-mechanics/src/recipes/
Assembly → create-contraptions/src/assembly/
Movimento → create-contraptions/src/movement/
Colisões → create-contraptions/src/collision/
Renderização móvel → create-contraptions/src/render/
```

A estrutura acima é ilustrativa. Adapte-a conforme a implementação real.

Regras de desenvolvimento:

- Cada sistema deve possuir responsabilidade clara.
- Evite arquivos gigantes com funcionalidades não relacionadas.
- Não divida código em dezenas de arquivos minúsculos sem necessidade.
- Não duplique lógica.
- Não mantenha versões antigas e novas do mesmo sistema simultaneamente sem justificativa.
- Documente interfaces públicas.
- Crie abstrações quando houver necessidade concreta.
- Evite abstrações prematuras.
- Utilize nomes claros e consistentes.
- Mantenha comentários úteis.
- Remova código morto quando for seguro.
- Não altere módulos não relacionados a uma tarefa sem necessidade.

Ao receber uma solicitação futura de atualização, primeiro consulte `AGENTS.md` e `docs/project-map.md`, identifique o módulo responsável e leia apenas os arquivos necessários e suas dependências relevantes.

Atualize o índice quando a estrutura mudar.

---

## 30. Testes obrigatórios

Crie:

- Testes unitários.
- Testes de integração.
- Testes de regressão.
- Testes de persistência.
- Testes de compatibilidade com Horizon.
- Testes de desempenho.
- Cenários de gameplay reproduzíveis.

### Testes de Core

- Water Wheel conectada a shaft.
- Engrenagem pequena conectada a grande.
- Sentidos de rotação.
- Mudança de RPM.
- Sobrecarga de rede.
- Recuperação após sobrecarga.
- Remoção de eixos.
- Divisão de redes.
- União de redes.
- Fontes incompatíveis.
- Ciclos.
- Clutch controlada por redstone.
- Chunk carregado e descarregado.

### Testes de Mechanics

- Pressagem de minério em sheets.
- Crushing Wheels esmagando raw ore.
- Encased Fan lavando crushed ore.
- Esteira transportando itens.
- Funil transferindo produtos.
- Produção automatizada de lingotes.
- Inventários cheios.
- Falta de energia.
- Reinício durante processamento.
- Probabilidades de subprodutos.
- Prevenção de duplicação.

### Testes de Contraptions

- Montagem de plataforma.
- Rotação por Mechanical Bearing.
- Desmontagem.
- Moinho de vento personalizado.
- Extensão de Mechanical Piston.
- Movimento vertical por Rope Pulley.
- Colisão com obstáculos.
- Persistência durante movimento.
- Inventários móveis.
- Colheita automatizada.
- Mineração automatizada.
- Recuperação após reiniciar o jogo.

Quando viável, utilize o Create original como referência para testes diferenciais.

Não invente resultados de testes.

Não declare que uma funcionalidade foi validada em jogo se apenas os testes unitários foram executados.

---

## 31. Benchmarks

Crie benchmarks reproduzíveis para:

### Redes cinéticas

- 100 componentes.
- 1.000 componentes.
- 10.000 componentes.

Meça:

- Tempo de criação da rede.
- Tempo de atualização após mudança.
- Tempo de simulação por tick.
- Consumo de memória.
- Alocações.
- Tempo de propagação.

### Automação industrial

- 100 máquinas.
- 1.000 máquinas.
- 100 linhas de esteiras.
- Processamento simultâneo de minérios.
- Transferência de grandes quantidades de itens.

### Contraptions

- Estruturas de 100 blocos.
- Estruturas de 1.000 blocos.
- Estruturas maiores dentro dos limites configurados.

Meça:

- Tempo de assembly.
- Tempo de disassembly.
- Tempo de atualização.
- Custo de renderização.
- Custo de colisões.
- Uso de RAM.
- Frametime.

Não estabeleça metas irreais sem medições iniciais.

Primeiro obtenha um baseline, depois defina metas de otimização.

O projeto deve priorizar boa estabilidade de frametime, TPS e consumo de memória, e não apenas FPS médio.

---

# PLANO DE IMPLEMENTAÇÃO

## 32. Desenvolvimento por fases

Não tente desenvolver o projeto inteiro em uma única alteração.

Execute as fases de forma incremental e mantenha o projeto compilável durante o desenvolvimento.

### Fase 0 — Auditoria e preparação

- Investigar Horizon.
- Investigar Create original.
- Investigar Create for Horizon.
- Verificar licenças.
- Verificar compatibilidade.
- Escolher uma revisão suportada do Horizon.
- Fixar versões das dependências.
- Compilar a implementação existente, quando aplicável.
- Identificar funcionalidades reutilizáveis.
- Identificar hooks necessários.
- Criar documentação de arquitetura.
- Criar documentação de lacunas.
- Criar roadmap.
- Criar AGENTS.md.
- Criar project-map.md.

Entregável: base técnica validada, projeto compilável e mapa das funcionalidades existentes e ausentes.

### Fase 1 — Create Core

- Implementar ou adaptar redes cinéticas.
- Water Wheel.
- Large Water Wheel.
- Shaft.
- Cogwheel.
- Large Cogwheel.
- Gearbox.
- Creative Motor.
- RPM.
- Stress.
- Renderização rotacional.

Entregável: rede mecânica funcional dentro do Horizon.

### Fase 2 — Core completo

- Clutch.
- Gearshift.
- Chain Drive.
- Rotation Speed Controller.
- Hand Crank.
- Wrench.
- Goggles.
- Receitas básicas.
- Persistência.
- Testes de redes.

Entregável: sistema mecânico confiável para alimentar os outros módulos.

### Fase 3 — Mechanics básico

- Mechanical Belt.
- Depot.
- Funnels.
- Mechanical Press.
- Millstone.
- Crushing Wheels.
- Encased Fan.
- Sistema de receitas.

Entregável: primeira linha industrial completa.

### Fase 4 — Processamento de minérios

- Crushing.
- Washing.
- Bulk Blasting.
- Bulk Smoking.
- Bulk Haunting.
- Crushed ores.
- Nuggets.
- Subprodutos.
- Automação.
- Balanceamento conforme Create original.

Entregável: fábrica automática de processamento e multiplicação de minérios.

### Fase 5 — Mechanics expandido

- Basin.
- Mixer.
- Drill.
- Saw.
- Harvester.
- Deployer.
- Blaze Burner.
- Mechanical Crafter.
- Chutes.
- Tunnels.

Entregável: experiência survival industrial básica completa.

### Fase 6 — Fundação de Contraptions

- Estrutura móvel.
- Assembly.
- Disassembly.
- Super Glue.
- Chassis.
- Renderização móvel.
- Transformações.
- Mechanical Bearing.

Entregável: estrutura personalizada girando de forma funcional.

### Fase 7 — Windmills

- Windmill Bearing.
- Sails.
- Estruturas personalizadas.
- Geração de energia cinética.
- Integração com redes.
- Persistência.

Entregável: moinho de vento personalizado alimentando uma fábrica real.

### Fase 8 — Contraptions lineares

- Mechanical Piston.
- Sticky Mechanical Piston.
- Rope Pulley.
- Gantry.
- Colisões.
- Sequenced Gearshift.

Entregável: pontes, portões, plataformas e máquinas móveis.

### Fase 9 — Contraptions produtivas

- Drill móvel.
- Saw móvel.
- Harvester móvel.
- Deployer móvel.
- Inventários móveis.
- Portable Storage Interface.

Entregável: fazendas automáticas giratórias e máquinas de mineração móveis.

### Fase 10 — Polimento e otimização

- Benchmarks.
- Profiling.
- Otimização.
- Correção de bugs.
- Compatibilidade multiplayer.
- Modelos.
- Texturas.
- Sons.
- Partículas.
- Interfaces.
- Traduções.
- Documentação.
- Empacotamento.

Entregável: versão jogável e distribuível do Create Reborn.

### Fase 11 — Expansões futuras

- Fluidos avançados.
- Steam Engines.
- Mechanical Arms.
- Sequenced Assembly.
- Elevadores avançados.
- Trens.
- Ponder.
- Integrações com outros mods.

Essas expansões não devem atrasar a primeira versão jogável.

---

## 33. Empacotamento e instalação

Investigue o formato real de empacotamento do Horizon.

O objetivo é permitir instalação dos módulos pelo Horizon Launcher.

Se o sistema suportar dependências entre mods nativos:

- Distribuir Create Core separadamente.
- Distribuir Create Mechanics com dependência de Core.
- Distribuir Create Contraptions com dependência de Core.
- Permitir instalação dos três módulos juntos.

Se ainda não houver suporte adequado para dependências entre bundles nativos:

- Manter os três módulos separados internamente.
- Criar um bundle único `.hnmod`.
- Disponibilizar configurações ou recursos de compilação para ativar ou desativar os módulos.
- Documentar a limitação.
- Preservar a possibilidade de distribuição independente no futuro.

Crie comandos ou scripts para:

- Preparar ambiente.
- Verificar compatibilidade.
- Compilar.
- Executar testes.
- Executar benchmarks.
- Empacotar.
- Instalar em uma instância local de desenvolvimento.

Utilize os mecanismos reais do Horizon e do registro oficial de mods.

Não invente comandos de launcher ou ferramentas de empacotamento.

Crie documentação para Windows e, quando possível, Linux.

Fixe as revisões do Horizon e Horizon Mixins necessárias para reproduzir o build.

---

## 34. Critérios de conclusão da primeira versão

A primeira versão considerada jogável deverá permitir:

1. Criar um mundo survival.
2. Obter os materiais necessários.
3. Construir uma Water Wheel.
4. Conectar shafts e cogwheels.
5. Alterar RPM com engrenagens.
6. Alimentar uma Mechanical Press.
7. Construir uma Crushing Wheel dupla.
8. Esmagar minérios.
9. Lavar crushed ores com Encased Fan.
10. Transportar itens usando Mechanical Belts.
11. Automatizar entrada e saída com funis.
12. Produzir recursos industriais e ligas.
13. Construir um moinho personalizado.
14. Utilizar o moinho para alimentar as máquinas.
15. Criar uma plataforma giratória.
16. Montar uma fazenda com Mechanical Harvesters.
17. Armazenar produtos automaticamente.
18. Utilizar Mechanical Pistons para mover estruturas.
19. Fechar o jogo e reabrir o mundo sem perder máquinas, inventários ou contraptions.
20. Jogar sem erros críticos, duplicação de itens ou corrupção do mundo.

A experiência deve ser semelhante ao Create básico, mesmo sem funcionalidades avançadas como trens.

---

## 35. Regras de execução do Codex

Você é responsável por atuar como engenheiro de software Rust, desenvolvedor de engines de jogos e arquiteto técnico deste projeto.

Não se limite a fornecer sugestões, pseudocódigo ou planos. Desenvolva os arquivos reais, compile o código, execute testes e implemente funcionalidades.

Entretanto:

- Não tente implementar tudo simultaneamente.
- Não declare funcionalidades concluídas sem evidência.
- Não invente resultados de compilação.
- Não invente APIs do Horizon.
- Não utilize placeholders como substitutos de funcionalidades finais.
- Não esconda erros de compilação.
- Não remova funcionalidades existentes para fazer testes passarem.
- Não modifique indiscriminadamente o código do Horizon.
- Não atualize dependências sem necessidade.
- Não utilize recursos externos não confiáveis sem verificar sua origem.
- Não execute código nativo desconhecido sem revisar sua procedência.
- Não faça alterações destrutivas em mundos ou instalações existentes.

Se uma API necessária não existir:

1. Identifique precisamente a limitação.
2. Localize o ponto de integração real na engine.
3. Determine se existe um hook ou mixin apropriado.
4. Proponha a menor alteração necessária.
5. Isole a adaptação.
6. Implemente testes.
7. Documente a dependência e a compatibilidade.

Ao concluir cada fase, registre:

- Funcionalidades implementadas.
- Arquivos modificados.
- Testes executados.
- Resultados reais.
- Limitações.
- Próximos passos.
- Pendências técnicas.

Mantenha o roadmap atualizado.

Use Git com alterações pequenas e coerentes, preservando o histórico do projeto original quando houver fork.

Se tiver acesso a uma cópia local do projeto, trabalhe nela diretamente.

Se o workspace estiver vazio, prepare o projeto e obtenha os repositórios necessários quando o ambiente permitir.

Se houver bloqueios de acesso à rede, dependências, compilação ou recursos locais, registre precisamente o bloqueio e execute todo o trabalho restante que for possível sem inventar resultados.

---

## 36. Primeira tarefa a executar agora

Comece imediatamente pela Fase 0.

1. Analise o conteúdo atual do workspace.
2. Identifique se já existe código do Horizon ou Create for Horizon.
3. Localize a implementação pública do Create for Horizon.
4. Analise sua arquitetura e licença.
5. Verifique a compatibilidade com o Horizon atual.
6. Compile a base existente ou uma revisão comprovadamente compatível.
7. Gere uma matriz de funcionalidades existentes versus funcionalidades desejadas.
8. Identifique os sistemas que podem ser reaproveitados.
9. Crie o Cargo workspace e a estrutura modular adequada.
10. Crie a documentação de arquitetura.
11. Crie AGENTS.md, project-map.md, compatibility.md e roadmap.md.
12. Implemente os primeiros testes de integração.
13. Inicie a Fase 1 assim que os requisitos técnicos estiverem validados.

Priorize preservar as funcionalidades já existentes na implementação Create for Horizon.

Não quero somente uma resposta explicando o que você faria. Quero que você execute o trabalho no projeto e entregue uma base real, organizada, compilável e pronta para evolução.

O objetivo final é ter um Create nativo em Rust para Horizon, dividido em Core, Mechanics e Contraptions, otimizado, modular, expansível e capaz de oferecer a experiência industrial mecânica do Create original.
