#include <cstring>

#include "CBaseListener.h"
#include "CLexer.h"
#include "CParser.h"
#include "antlr4-runtime.h"

typedef char *(*RsGetStr)(size_t len);
typedef void (*RsEmitEvent)(char *tag, char *text);

class EventListener : public CBaseListener {
   public:
    EventListener(RsGetStr rsGetStr, RsEmitEvent rsEmitEvent)
        : rsGetStr(rsGetStr), rsEmitEvent(rsEmitEvent) {}

    void exitPrimaryExpression(
        CParser::PrimaryExpressionContext *ctx) override {
        if (ctx->Identifier() || ctx->Constant()) {
            this->emitEvent("ExitPrimaryExpression", ctx->getText().c_str());
        }
    }

    void exitPostfixExpression(
        CParser::PostfixExpressionContext *ctx) override {
        if (ctx->LeftParen() || ctx->argumentExpressionList()) {
            this->emitEvent("ExitPostfixExpression", "");
        }
    }

    void exitArgumentExpressionList(
        CParser::ArgumentExpressionListContext *ctx) override {
        this->emitEvent("ExitArgumentExpressionList", "");
    }

    void exitUnaryExpression(CParser::UnaryExpressionContext *ctx) override {
        if (auto op = ctx->unaryOperator(); op && op->Not()) {
            this->emitEvent("ExitUnaryExpression", "!");
        }
    }

    void exitMultiplicativeExpression(
        CParser::MultiplicativeExpressionContext *ctx) override {
        if (ctx->Star()) {
            this->emitEvent("ExitMultiplicativeExpression", "*");
        } else if (ctx->Div()) {
            this->emitEvent("ExitMultiplicativeExpression", "/");
        }
    }

    void exitAdditiveExpression(
        CParser::AdditiveExpressionContext *ctx) override {
        if (ctx->Plus()) {
            this->emitEvent("ExitAdditiveExpression", "+");
        } else if (ctx->Minus()) {
            this->emitEvent("ExitAdditiveExpression", "-");
        }
    }

    void exitRelationalExpression(
        CParser::RelationalExpressionContext *ctx) override {
        if (ctx->Less()) {
            this->emitEvent("ExitRelationalExpression", "<");
        } else if (ctx->Greater()) {
            this->emitEvent("ExitRelationalExpression", ">");
        } else if (ctx->LessEqual()) {
            this->emitEvent("ExitRelationalExpression", "<=");
        } else if (ctx->GreaterEqual()) {
            this->emitEvent("ExitRelationalExpression", ">=");
        }
    }

    void exitEqualityExpression(
        CParser::EqualityExpressionContext *ctx) override {
        if (ctx->Equal()) {
            this->emitEvent("ExitEqualityExpression", "==");
        } else if (ctx->NotEqual()) {
            this->emitEvent("ExitEqualityExpression", "!=");
        }
    }

    void exitLogicalAndExpression(
        CParser::LogicalAndExpressionContext *ctx) override {
        if (ctx->AndAnd()) {
            this->emitEvent("ExitLogicalAndExpression", "&&");
        }
    }

    void exitLogicalOrExpression(
        CParser::LogicalOrExpressionContext *ctx) override {
        if (ctx->OrOr()) {
            this->emitEvent("ExitLogicalOrExpression", "||");
        }
    }

    void exitDeclaration(CParser::DeclarationContext *ctx) override {
        this->emitEvent("ExitDeclaration", ctx->initDeclaratorList()
                                               ->initDeclarator()
                                               ->declarator()
                                               ->directDeclarator()
                                               ->Identifier()
                                               ->getText()
                                               .c_str());
    }

    void enterCompoundStatement(
        CParser::CompoundStatementContext *ctx) override {
        this->emitEvent("EnterCompoundStatement", "");
    }

    void exitCompoundStatement(
        CParser::CompoundStatementContext *ctx) override {
        this->emitEvent("ExitCompoundStatement", "");
    }

    void exitExpressionStatement(
        CParser::ExpressionStatementContext *ctx) override {
        this->emitEvent("ExitExpressionStatement", "");
    }

    void exitSelectionStatement(
        CParser::SelectionStatementContext *ctx) override {
        if (ctx->If()) {
            if (ctx->Else()) {
                this->emitEvent("ExitSelectionStatement", "else");
            } else {
                this->emitEvent("ExitSelectionStatement", "");
            }
        }
    }

    void exitIterationStatement(
        CParser::IterationStatementContext *ctx) override {
        if (ctx->While()) {
            this->emitEvent("ExitIterationStatement", "");
        }
    }

    void exitJumpStatement(CParser::JumpStatementContext *ctx) override {
        if (ctx->Return()) {
            if (auto expr = ctx->expression()) {
                this->emitEvent("ExitJumpStatement", "expr");
            } else {
                this->emitEvent("ExitJumpStatement", "");
            }
        }
    }

    void exitFunctionDefinition(
        CParser::FunctionDefinitionContext *ctx) override {
        std::string sig;
        // Determine the return type.
        if (ctx->declarationSpecifiers()
                ->declarationSpecifier(0)
                ->typeSpecifier()
                ->Void()) {
            sig.append("void");
        } else {
            sig.append("int");
        }
        // Determine the function name.
        sig.push_back(' ');
        auto name = ctx->declarator()
                        ->directDeclarator()
                        ->directDeclarator()
                        ->Identifier()
                        ->getText();
        sig.append(name);
        // Determine the argument list.
        if (auto param_list =
                ctx->declarator()->directDeclarator()->parameterTypeList()) {
            auto parameter = param_list->parameterList();
            while (parameter) {
                auto param = parameter->parameterDeclaration()
                                 ->declarator()
                                 ->directDeclarator()
                                 ->getText();
                sig.push_back(' ');
                sig.append(param);
                parameter = parameter->parameterList();
            }
        }
        this->emitEvent("ExitFunctionDefinition", sig.c_str());
    }

   private:
    RsGetStr rsGetStr;
    RsEmitEvent rsEmitEvent;

    void emitEvent(const char *tag, const char *text) {
        auto rsTag{this->rsGetStr(std::strlen(tag))};
        std::strcpy(rsTag, tag);
        auto rsText{this->rsGetStr(std::strlen(text))};
        std::strcpy(rsText, text);
        this->rsEmitEvent(rsTag, rsText);
    }
};

extern "C" char *_parse(char *source, RsGetStr rsGetStr,
                        RsEmitEvent rsEmitEvent) {
    antlr4::ANTLRInputStream input{source};
    CLexer lexer{&input};
    antlr4::CommonTokenStream tokens{&lexer};
    CParser parser{&tokens};
    antlr4::tree::ParseTree *tree{parser.compilationUnit()};
    EventListener listener{rsGetStr, rsEmitEvent};
    antlr4::tree::ParseTreeWalker::DEFAULT.walk(&listener, tree);
    return source;
}
