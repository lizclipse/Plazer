import { fireEvent, render } from "@solidjs/testing-library";
import Counter from "./Counter";

describe("<Counter />", () => {
  it("increments value", async () => {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const { queryByRole, unmount } = render(() => <Counter />);
    // eslint-disable-next-line @typescript-eslint/await-thenable
    const button = (await queryByRole("button")) as HTMLButtonElement;
    expect(button).toBeInTheDocument();
    expect(button).toHaveTextContent(/Clicks: 0/);
    fireEvent.click(button);
    expect(button).toHaveTextContent(/Clicks: 1/);
    unmount();
  });

  it("renders 1", () => {
    const { container, unmount } = render(() => <Counter />);
    expect(container).toMatchSnapshot();
    unmount();
  });
});
