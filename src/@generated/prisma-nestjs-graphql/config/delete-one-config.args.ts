import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigWhereUniqueInput } from './config-where-unique.input';

@ArgsType()
export class DeleteOneConfigArgs {

    @Field(() => ConfigWhereUniqueInput, {nullable:false})
    where!: ConfigWhereUniqueInput;
}
