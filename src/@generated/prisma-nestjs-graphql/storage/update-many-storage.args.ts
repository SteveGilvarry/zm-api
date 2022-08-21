import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageUpdateManyMutationInput } from './storage-update-many-mutation.input';
import { Type } from 'class-transformer';
import { StorageWhereInput } from './storage-where.input';

@ArgsType()
export class UpdateManyStorageArgs {

    @Field(() => StorageUpdateManyMutationInput, {nullable:false})
    @Type(() => StorageUpdateManyMutationInput)
    data!: StorageUpdateManyMutationInput;

    @Field(() => StorageWhereInput, {nullable:true})
    @Type(() => StorageWhereInput)
    where?: StorageWhereInput;
}
