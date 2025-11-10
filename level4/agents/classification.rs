pub trait Agent {
    fn name(&self) -> &str;
    fn act(&self, input: &str, context: &Notebook) -> AgentOutput;
}

pub struct ClassificationAgent;
pub struct ReasoningAgent;
pub struct ActionAgent;
pub struct GraphRetriever;

pub fn glm_reasoning(question: &str) -> String {
    let mut notebook = Notebook::new();
    let classifier = ClassificationAgent;
    let reasoner = ReasoningAgent;
    let actor = ActionAgent;
    let retriever = GraphRetriever;

    match classifier.act(question, &notebook).output_type {
        OutputType::Deterministic => {
            let code = actor.act(question, &notebook).code;
            let result = retriever.execute(&code);
            result
        }
        OutputType::NonDeterministic => {
            loop {
                let reasoning = reasoner.act(question, &notebook);
                if reasoning.is_complete {
                    return reasoning.answer;
                }
                let code = actor.act(&reasoning.missing_info, &notebook).code;
                let facts = retriever.execute(&code);
                notebook.update(facts);
            }
        }
    }
}
