import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageWhereInput } from './storage-where.input';

@ArgsType()
export class DeleteManyStorageArgs {

    @Field(() => StorageWhereInput, {nullable:true})
    where?: StorageWhereInput;
}
