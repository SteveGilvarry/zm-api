import { Test, TestingModule } from '@nestjs/testing';
import { ModelsResolver } from './models.resolver';
import { ModelsService } from './models.service';

describe('ModelsResolver', () => {
  let resolver: ModelsResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ModelsResolver, ModelsService],
    }).compile();

    resolver = module.get<ModelsResolver>(ModelsResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
