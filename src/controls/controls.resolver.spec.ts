import { Test, TestingModule } from '@nestjs/testing';
import { ControlsResolver } from './controls.resolver';
import { ControlsService } from './controls.service';

describe('ControlsResolver', () => {
  let resolver: ControlsResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ControlsResolver, ControlsService],
    }).compile();

    resolver = module.get<ControlsResolver>(ControlsResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
