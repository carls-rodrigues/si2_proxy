# SI Proxy - Web Proxy com Controle de Conteúdo

Trabalho do Primeiro Bimestre para a disciplina de **Sistemas para Internet 2** (Prof. Dr. André Prisco Vargas).

Este projeto consiste em um servidor proxy web funcional com controle de acesso, filtro de conteúdo em tempo real e logging. O proxy foi projetado para atuar como intermediário em requisições HTTP, demonstrando conceitos estudados em rede, como manipulação de cabeçalhos, roteamento, inspeção e substituição de corpo de texto.

## 🚀 Funcionalidades

1. **Modo Transparente**: Funciona como um intermediário padrão. Recebe a requisição (suportando qualquer método HTTP graças à utilização flexível de *bytes* e manipulação de *HeaderMap*), encaminha para o servidor de origem e repassa a resposta ao cliente, mantendo os cabeçalhos intocados.
2. **Bloqueio de Sites**: Rejeita o acesso a domínios predefinidos na lista negra e retorna uma página customizada de bloqueio.
3. **Filtro de Palavrões (Conteúdo)**: Se a resposta possuir o Content-Type `text/html`, realiza uma varredura *case-insensitive* e substitui palavras proibidas.
4. **Log de Acessos**: Grava todas as transações em `logs/log.json` informando *timestamp*, *url* e a *ação* tomada (permitido, bloqueado ou filtrado).

---

## 🛠️ Escolha da Tecnologia (Justificativa)

A tecnologia escolhida para a implementação deste servidor foi a linguagem **Rust**, orquestrada pelo runtime assíncrono **Tokio**, utilizando o framework web **Axum** para as rotas e o client HTTP **reqwest** para realizar as requisições ao servidor de destino.

**Por que Rust e não Python/Flask?**
Optou-se pelo Rust pelo desafio técnico e por suas garantias de segurança de memória e alta performance. Um servidor proxy lida intensamente com I/O (abertura e fechamento contínuo de conexões, leitura de *streams* de dados); a concorrência assíncrona do Rust associada ao `tokio` proporciona uma fundação extremamente eficiente que consome baixíssima memória comparada ao Python.

**Vantagens em relação às alternativas:**
* **Tipagem forte e *Ownership*:** Muitos erros clássicos de concorrência ou variáveis não inicializadas foram pegos logo na compilação.
* **Performance bruta:** O tempo de repasse da requisição é incrivelmente baixo.
* **Ecossistema:** Bibliotecas modernas (como `axum`, `reqwest` e `regex`) permitiram desenvolver a lógica complexa de manipulação HTTP de forma limpa.

**Dificuldades enfrentadas:**
* A curva de aprendizado inicial do gerenciamento de memória (lifetimes, referências) exige muito mais código *boilerplate* comparado ao Python.
* Lidar com o tipo de retorno dinâmico de `Body` e `Bytes` do Axum gerou atritos, exigindo atenção especial no momento de extrair e remontar as respostas (e repassar corretamente os *headers* e o *body* da requisição original, filtrando os cabeçalhos `hop-by-hop`).

---

## ⚙️ Instalação das Dependências

Sendo um projeto em Rust, a principal dependência é ter a *toolchain* do Rust instalada no ambiente (`rustc` e `cargo`).

1. Para instalar o Rust no Linux/macOS:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Clone o repositório:
   ```bash
   git clone <URL_DO_SEU_REPOSITORIO>
   cd si_proxy
   ```
3. (Opcional) Compile o projeto para garantir que as dependências do `Cargo.toml` (Axum, Tokio, Reqwest) foram baixadas com sucesso:
   ```bash
   cargo build
   ```

---

## 📝 Como configurar as Listas

As configurações de filtragem e bloqueio baseiam-se em dois arquivos locais na pasta `config/`. O formato exigido é estritamente JSON.

**1. Bloqueio de domínios (`config/blocked.json`)**
Adicione os domínios que deseja bloquear em formato de vetor de Strings. Exemplo:
```json
{
  "bloqueados": [
    "www.sitex.com",
    "redes-sociais.net"
  ]
}
```

**2. Substituição de palavras (`config/words.json`)**
Adicione os pares de chave (palavra a ser bloqueada) e valor (o termo substituto). A substituição é *case-insensitive*. Exemplo:
```json
{
  "merda": "macacos me mordam",
  "idiota": "ingênuo"
}
```

---

## ▶️ Como executar o Proxy

Para iniciar o servidor proxy, utilize o gerenciador do Rust:

```bash
cargo run
```
O console deverá exibir `SI Proxy rodando em http://localhost:5000`.

**Para testar o acesso via proxy**, adicione a URL de destino na rota raiz do servidor local:
```
http://localhost:5000/http://www.google.com
```

Você pode utilizar requisições avançadas via `curl` no terminal para testar fluxos `POST`:
```bash
curl -X POST "http://localhost:5000/http://httpbin.org/post" \
     -H "Content-Type: application/json" \
     -d '{"mensagem": "testando o proxy"}'
```

---

## 🤖 Transparência no uso de Inteligência Artificial

Ferramentas de Inteligência Artificial (assistentes de código baseados em LLMs) foram utilizadas ativamente no desenvolvimento deste trabalho, servindo primariamente como *Pair Programming* e tutoriais guiados.

* **O que foi gerado:** A inteligência artificial foi utilizada para sanar dúvidas quanto à manipulação de *Streams* e do Body do `reqwest` e `axum`. Dúvidas sobre como ignorar corretamente cabeçalhos do tipo *Hop-By-Hop* para não quebrar a transação também foram validadas com a IA.
* **O que foi modificado/entendido:** A IA atuou como revisora, identificando que na versão inicial o método `GET` havia sido cravado (hardcoded), e auxiliou na refatoração para que o *proxy* pudesse receber e repassar qualquer método (POST, PUT, etc.) enviando também o corpo da requisição do cliente original. Além disso, as dúvidas sobre falhas relativas à renderização de SPAs (arquivos Javascript estáticos caindo em rotas URL-Prefix) foram esclarecidas pela ferramenta.
* **Aprendizado:** A interação permitiu um aprendizado profundo não só na sintaxe e compilação do Rust, mas nos limites do modelo de proxy adotado e o quão minuciosas são as especificações do protocolo HTTP 1.1 e 2.0 sob a ótica de um intermediário.
