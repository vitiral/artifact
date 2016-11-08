import { Component } from '@angular/core';

import { Artifact } from './artifact/artifact.component';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent {
  title = 'app works!';
  checkFor = ["ngFor", "works"];

  artifact = new Artifact(
    {value: 'REQ-artifact'},
    'path',
    {value: 'text'},
    [{value: 'partof-1'}, {value: 'partof-2'}],
    ['parts-1', 'parts-2'],
    { row: 2, col: 2 }, 0, 0);
}
