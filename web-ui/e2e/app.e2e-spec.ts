import { WebUiPage } from './app.po';

describe('web-ui App', function() {
  let page: WebUiPage;

  beforeEach(() => {
    page = new WebUiPage();
  });

  it('should display message saying app works', () => {
    page.navigateTo();
    expect(page.getParagraphText()).toEqual('app works!');
  });
});
