# Quantum LIMIT-GRAPH Level 4 — GLM Architecture 

> 🧠 A modular, high-performance, multi-agent system for quantum-aligned graph reasoning and multilingual benchmarking.

This repository implements the **Level 4 maturity upgrade** of the [Quantum LIMIT-GRAPH](https://github.com/NurcholishAdam/Quantum-LIMIT-GRAPH) project. It features a **Rust-native GLM architecture** with multi-agent orchestration, vertex-centric KV-cache reuse, and streaming inference—designed for scalable, reproducible, and multilingual scientific reasoning.

---

## 🚀 Key Features

- **Multi-Agent GLM Architecture**  
  Modular agents for classification, reasoning, action generation, and retrieval—enabling compositional and adaptive reasoning.

- **Vertex-Centric KV-Cache Reuse**  
  Efficient memory reuse across graph nodes to reduce decoding latency and improve inference throughput.

- **Code-Based Retrieval via Action Agent**  
  Replaces prompt-based graph access with executable Rust logic for structured, reproducible retrieval.

- **Streaming Inference & Parallel Graph Access**  
  Async inference pipeline with concurrent graph traversal and real-time response streaming.

- **Rust-First Implementation**  
  Built for performance, safety, and extensibility using idiomatic Rust and async runtimes.

---

## 🧩 Architecture Overview
Coordinator Agent 
│ 
├── ClassificationAgent 
├── ReasoningAgent 
├── ActionAgent 
└── GraphRetriever


Each agent is implemented as a trait with composable logic. The Coordinator orchestrates reasoning loops and dynamically assembles pipelines based on task type (deterministic vs. non-deterministic).

---

## 📦 Repository Structure
. ├── level4/ # Core multi-agent logic in Rust 
  │ 
  ├── agents/ # Agent trait definitions and implementations 
  │ 
  ├── cache/ # Vertex-centric KV-cache manager 
  │ 
  ├── retriever/ # Code-based graph retrieval engine 
  │ 
  ├── coordinator.rs # GLM reasoning loop and agent orchestration 
  │ 
  └── lib.rs # Library entry point 
  ├── LICENSE # MIT License 
  └── README.md # Project overview and setup instructions

  
---

## 🛠️ Getting Started

### Prerequisites
- Rust (1.74+)
- Cargo
- Neo4j (optional, for graph backend)
- Hugging Face API key (optional, for LLM integration)

### Build & Run

```bash
git clone https://github.com/NurcholishAdam/Quantum-Limit-Graph-level4-GLM-Architecture.git
cd Quantum-Limit-Graph-level4-GLM-Architecture
cargo build --release
cargo run
```


### Example Use

```python
let question = "What is the most cited Indonesian NLP paper from 2021?";
let answer = glm_reasoning(question);
println!("Answer: {}", answer);
```



🤝 Contributing
We welcome contributors! Please see CONTRIBUTING.md (coming soon) for onboarding instructions, coding style, and benchmarking guidelines.

📜 License
This project is licensed under the MIT License.

🌐 Related Projects
Quantum LIMIT-GRAPH v2.4.0-NSN

✨ Acknowledgments
This project is part of a broader mission to democratize agentic retrieval, multilingual benchmarking, and quantum-aligned AI research—led by @NurcholishAdam.
