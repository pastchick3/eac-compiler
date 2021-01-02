#include "CBaseListener.h"
#include "CLexer.h"
#include "CParser.h"
#include "antlr4-runtime.h"
#include <fstream>
#include <vector>

extern "C" typedef struct {
  char *tag;
  const char *text;
} Event;

extern "C" typedef struct {
  Event *data;
  size_t len;
} Events;

class EventListener : public CBaseListener {
public:
  Event *getEventsPtr() { return this->events.data(); }

  size_t getEventsSize() { return this->events.size(); }

  void
  enterFunctionDefinition(CParser::FunctionDefinitionContext *ctx) override {
    Event event;
    event.tag = "EnterFunction";
    event.text = ctx->getText().c_str();
    this->events.push_back(event);
  }

private:
  std::vector<Event> events;
};

extern "C" Events _parse(char *path) {
  std::ifstream file;
  file.open(path);
  antlr4::ANTLRInputStream input(file);
  CLexer lexer(&input);
  antlr4::CommonTokenStream tokens(&lexer);
  CParser parser(&tokens);
  antlr4::tree::ParseTree *tree = parser.compilationUnit();
  EventListener listener;
  antlr4::tree::ParseTreeWalker::DEFAULT.walk(&listener, tree);
  std::cout << tree->toStringTree(&parser) << std::endl;
  file.close();
  Events events;
  events.data = listener.getEventsPtr();
  events.len = listener.getEventsSize();
  return events;
}
