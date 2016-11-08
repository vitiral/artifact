import { Component, Input, OnInit } from '@angular/core';

export class Loc {
    constructor(
        public row: number,
        public col: number,
    ) {}
}

// class that allows you to edit strings in place
export class StringRef {
    value: string;
}

const validArtNamePat = new RegExp(
    "(REQ|SPC|RSK|TST)-[A-Z0-9_-]+$");

const REQ = "REQ";
const RSK = "RSK";
const SPC = "SPC";
const TST = "TST";

export class Artifact {
  constructor(
    public name: StringRef,
    public path: string,
    public text: StringRef,
    public partof: StringRef[],
    public parts: string[],
    public loc: Loc,
    public completed: number,
    public tested: number,
  ) {}
}

@Component({
  selector: 'artifact',
  templateUrl: './artifact.component.html',
  styleUrls: ['./artifact.component.css']
})
export class ArtifactComponent implements OnInit {
  @Input() model: Artifact;
  newPartof: string = "";
  errors: string[] = [];
  artNameErrorMsg = `\
    Artifact must start with "REQ-", "RSK-", "SPC-" or "TST-" \
    and must only contain alpha-numeric ASCII, '-' and/or '_' \
    characters (spaces are ignored)`

  constructor() { }

  ngOnInit() {
  }

  updatePartof() {
      console.log("updating partof");
      this.model.partof = this.model.partof.filter(
          (p) => p.value);
  }

  addNewPartof(): void {
      if (this.newPartof) {
          this.model.partof.push({value: this.newPartof})
      }
      this.newPartof = "";
  }

  artNameValidFull(artname: string): boolean {
      return this.artNameValid(artname) && !this.partofNameError(artname);
  }

  artNameValid(artname: string): boolean {
    // ignore empty values, they will get cleaned up
    if (!artname) {
        return true;
    }
    artname = artname.replace(" ", "").toUpperCase();
    return validArtNamePat.test(artname);
  }

  partofNameError(artname: string): string {
      // TODO: check whether the types match
      var that_type = artname.slice(0, 3).toUpperCase();
      // if that_type is invalid, ignore (other error will catch)
      if ([REQ, RSK, SPC, TST].indexOf(that_type) < 0) {
          return "";
      }
      var this_type = this.model.name.value.slice(0, 3).toUpperCase();
      if (this_type == REQ && that_type != REQ) {
          return "REQ can only be partof REQ";
      }
      if (this_type == RSK && [REQ, RSK].indexOf(that_type) < 0) {
          return "RSK can only be partof REQ or RSK";
      }
      if (this_type == SPC && [REQ, SPC].indexOf(that_type) < 0) {
          return "SPC can only be partof REQ or SPC";
      }
      if (this_type == TST && [RSK, SPC, TST].indexOf(that_type) < 0) {
          return "TST can only be partof RSK, SPC or TST";
      }
      return "";
  }

  valid(): boolean {
      this.updatePartof();
      var errors = [];
      if (!this.artNameValidFull(this.model.name.value)) {
          return false;
      }
      for (let part of this.model.partof) {
          if (!this.artNameValidFull(part.value)) {
              return false;
          }
      }
      if (this.newPartof && !this.artNameValidFull(this.newPartof)) {
          return false;
      }
      return true;
  }

  switchFocus(element: any) {
      element.focus();
  }

  submit(): void {
      this.addNewPartof()
      this.updatePartof()
      // TODO: make request to server
  }
}
