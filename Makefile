
ifneq ('', $(shell type gshuf))
SHUF := gshuf
else
SHUF := shuf
endif

words: itwêwina
	look . | $(SHUF) -n $(shell wc -l $< | awk '{print $$1}')\
		| sort -f > $@
