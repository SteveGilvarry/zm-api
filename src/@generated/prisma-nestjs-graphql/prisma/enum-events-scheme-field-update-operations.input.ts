import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Events_Scheme } from './events-scheme.enum';

@InputType()
export class EnumEvents_SchemeFieldUpdateOperationsInput {

    @Field(() => Events_Scheme, {nullable:true})
    set?: keyof typeof Events_Scheme;
}
