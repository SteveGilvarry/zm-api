import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageWhereUniqueInput } from './storage-where-unique.input';

@ArgsType()
export class FindUniqueStorageArgs {

    @Field(() => StorageWhereUniqueInput, {nullable:false})
    where!: StorageWhereUniqueInput;
}
