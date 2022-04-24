import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10UpdateManyMutationInput } from './triggers-x-10-update-many-mutation.input';
import { TriggersX10WhereInput } from './triggers-x-10-where.input';

@ArgsType()
export class UpdateManyTriggersX10Args {

    @Field(() => TriggersX10UpdateManyMutationInput, {nullable:false})
    data!: TriggersX10UpdateManyMutationInput;

    @Field(() => TriggersX10WhereInput, {nullable:true})
    where?: TriggersX10WhereInput;
}
