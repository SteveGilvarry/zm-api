import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigCreateInput } from './config-create.input';

@ArgsType()
export class CreateOneConfigArgs {

    @Field(() => ConfigCreateInput, {nullable:false})
    data!: ConfigCreateInput;
}
