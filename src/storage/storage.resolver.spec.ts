import { Test, TestingModule } from '@nestjs/testing';
import { StorageResolver } from './storage.resolver';
import { StorageService } from './storage.service';

describe('StorageResolver', () => {
  let resolver: StorageResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [StorageResolver, StorageService],
    }).compile();

    resolver = module.get<StorageResolver>(StorageResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
